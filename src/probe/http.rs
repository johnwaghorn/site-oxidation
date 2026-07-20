use crate::models::site::SiteStatus;
use crate::security::resolver::{ResolveError, resolve_public_addrs, warn_probe_private_host};
use reqwest::{Client, StatusCode};
use std::time::Duration;

const MAX_ERROR_MESSAGE_CHARS: usize = 500;
const EXPECTED_TEXT_MISSING_MESSAGE: &str = "Expected response text was not found";
pub(crate) const PRIVATE_IP_BLOCKED_MESSAGE: &str =
    "Request blocked because the target resolves to a private or internal IP address";
const RESPONSE_BODY_READ_ERROR_PREFIX: &str = "Failed to read response body:";
pub(crate) const UNKNOWN_PROBE_ERROR_MESSAGE: &str = "no error message";

pub struct CheckExpectation {
    pub expected_status: u16,
    pub expected_text: Option<String>,
}

pub struct ProbeResult {
    pub status: SiteStatus,
    pub status_code: Option<StatusCode>,
    pub latency_ms: Option<u128>,
    pub error_message: Option<String>,
}

fn bounded_error_message(message: &str) -> String {
    message.chars().take(MAX_ERROR_MESSAGE_CHARS).collect()
}

async fn validate_probe_response(
    response: reqwest::Response,
    check: &CheckExpectation,
) -> Result<(), String> {
    let actual_status = response.status().as_u16();
    if actual_status != check.expected_status {
        return Err(format!(
            "Expected HTTP status {}, received {actual_status}",
            check.expected_status
        ));
    }
    let Some(expected_text) = &check.expected_text else {
        return Ok(());
    };
    let body = response.text().await.map_err(|error| {
        bounded_error_message(&format!("{RESPONSE_BODY_READ_ERROR_PREFIX} {error}"))
    })?;
    if !body.contains(expected_text) {
        return Err(EXPECTED_TEXT_MISSING_MESSAGE.to_owned());
    }
    Ok(())
}

pub async fn check_url(
    client: &Client,
    url: &str,
    check: &CheckExpectation,
    timeout_secs: u64,
    allow_private_ips: bool,
) -> ProbeResult {
    if !allow_private_ips
        && let Ok(parsed) = reqwest::Url::parse(url)
        && let Some(host) = parsed.host_str()
    {
        let port = parsed.port_or_known_default().unwrap_or(443);
        // A DNS failure falls through so reqwest can surface the real error.
        if let Err(ResolveError::PrivateIp { .. }) = resolve_public_addrs(host, port)
            .await
            .inspect_err(warn_probe_private_host)
        {
            return ProbeResult {
                status: SiteStatus::Blocked,
                status_code: None,
                latency_ms: None,
                error_message: Some(PRIVATE_IP_BLOCKED_MESSAGE.to_owned()),
            };
        }
    }
    let start = std::time::Instant::now();
    match client
        .get(url)
        .timeout(Duration::from_secs(timeout_secs))
        .send()
        .await
    {
        Ok(response) => {
            let status_code = response.status();
            let latency_ms = start.elapsed().as_millis();
            let error_message = validate_probe_response(response, check).await.err();
            ProbeResult {
                status: if error_message.is_none() {
                    SiteStatus::Up
                } else {
                    SiteStatus::Down
                },
                status_code: Some(status_code),
                latency_ms: Some(latency_ms),
                error_message,
            }
        }
        Err(error) => ProbeResult {
            status: SiteStatus::Down,
            status_code: None,
            latency_ms: None,
            error_message: Some(bounded_error_message(&error.to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::resolver::SafeResolver;
    use crate::tests::TestHttpServer;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn truncated_response_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 1024];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort",
                )
                .await
                .unwrap();
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_check_url_applies_literal_private_ip_policy() {
        let server = TestHttpServer::start().await;
        let client = Client::new();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let result = check_url(&client, server.base_url(), &check, 1, false).await;
        assert!(
            result.status.is_blocked(),
            "literal private IP should mark site as Blocked"
        );
        assert!(logs_contain(
            "Blocked '127.0.0.1': resolves to private/internal IP"
        ));
        assert!(logs_contain("PROBE_ALLOW_PRIVATE_IPS=true"));
        assert_eq!(
            result.error_message.as_deref(),
            Some(PRIVATE_IP_BLOCKED_MESSAGE)
        );
        let allowed = check_url(&client, server.base_url(), &check, 1, true).await;
        assert!(allowed.status.is_up());
        assert_eq!(allowed.status_code, Some(StatusCode::OK));
        assert_eq!(server.request_count(), 1);
    }

    #[tokio::test]
    async fn test_check_url_applies_private_hostname_policy() {
        let server = TestHttpServer::start().await;
        let blocked_client = Client::builder()
            .dns_resolver(Arc::new(SafeResolver {
                allow_private: false,
            }))
            .build()
            .unwrap();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let url = format!("http://localhost:{}", server.port());
        let result = check_url(&blocked_client, &url, &check, 1, false).await;
        assert!(
            result.status.is_blocked(),
            "hostname resolving to a private IP should mark site as Blocked"
        );
        let allowed_client = Client::builder()
            .dns_resolver(Arc::new(SafeResolver {
                allow_private: true,
            }))
            .build()
            .unwrap();
        let allowed = check_url(&allowed_client, &url, &check, 1, true).await;
        assert!(allowed.status.is_up());
        assert_eq!(allowed.status_code, Some(StatusCode::OK));
        assert_eq!(server.request_count(), 1);
    }

    #[tokio::test]
    async fn test_check_url_validates_status_and_expected_text() {
        let server = TestHttpServer::start().await;
        let client = Client::new();
        let cases = [
            (
                204,
                None,
                SiteStatus::Down,
                Some("Expected HTTP status 204, received 200"),
            ),
            (
                200,
                Some("healthy"),
                SiteStatus::Down,
                Some(EXPECTED_TEXT_MISSING_MESSAGE),
            ),
            (200, Some("ok"), SiteStatus::Up, None),
        ];
        for (expected_status, expected_text, status, error_message) in cases {
            let check = CheckExpectation {
                expected_status,
                expected_text: expected_text.map(str::to_owned),
            };
            let result = check_url(&client, server.base_url(), &check, 1, true).await;
            assert_eq!(result.status, status);
            assert_eq!(result.error_message.as_deref(), error_message);
        }
    }

    #[tokio::test]
    async fn test_check_url_reports_response_body_read_failure() {
        let url = truncated_response_url().await;
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: Some("healthy".to_owned()),
        };
        let result = check_url(&Client::new(), &url, &check, 1, true).await;
        assert!(result.status.is_down());
        assert!(
            result
                .error_message
                .as_deref()
                .is_some_and(|message| message.starts_with(RESPONSE_BODY_READ_ERROR_PREFIX))
        );
    }
}
