use crate::models::site::SiteStatus;
use crate::security::resolver::{ResolveError, resolve_public_addrs, warn_probe_private_host};
use reqwest::{Client, StatusCode};
use std::time::Duration;

const MAX_ERROR_MESSAGE_CHARS: usize = 500;
const EXPECTED_TEXT_MISSING_MESSAGE: &str = "Expected response text was not found";
pub(crate) const PRIVATE_IP_BLOCKED_MESSAGE: &str =
    "Request blocked because the target resolves to a private or internal IP address";
const RESPONSE_BODY_LIMIT_EXCEEDED_PREFIX: &str = "Response body exceeded configured limit";
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

async fn validate_expected_text(
    mut response: reqwest::Response,
    expected_text: &str,
    body_size_limit_bytes: usize,
) -> Result<(), String> {
    let needle = expected_text.as_bytes();
    if needle.is_empty() {
        return Ok(());
    }
    let mut body = Vec::with_capacity(body_size_limit_bytes.min(8_192));
    while let Some(chunk) = response.chunk().await.map_err(|error| {
        bounded_error_message(&format!("{RESPONSE_BODY_READ_ERROR_PREFIX} {error}"))
    })? {
        if chunk.is_empty() {
            continue;
        }
        let remaining = body_size_limit_bytes.saturating_sub(body.len());
        let search_from = body.len().saturating_sub(needle.len().saturating_sub(1));
        let exceeds_limit = chunk.len() > remaining;
        body.extend(chunk.iter().copied().take(remaining));
        if body
            .get(search_from..)
            .is_some_and(|bytes| bytes.windows(needle.len()).any(|window| window == needle))
        {
            return Ok(());
        }
        if exceeds_limit {
            return Err(format!(
                "{RESPONSE_BODY_LIMIT_EXCEEDED_PREFIX} of {body_size_limit_bytes} bytes before expected text was found"
            ));
        }
    }
    Err(EXPECTED_TEXT_MISSING_MESSAGE.to_owned())
}

async fn validate_probe_response(
    response: reqwest::Response,
    check: &CheckExpectation,
    body_size_limit_bytes: usize,
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
    validate_expected_text(response, expected_text, body_size_limit_bytes).await
}

pub async fn check_url(
    client: &Client,
    url: &str,
    check: &CheckExpectation,
    timeout_secs: u64,
    body_size_limit_bytes: usize,
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
            let error_message = validate_probe_response(response, check, body_size_limit_bytes)
                .await
                .err();
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

    const TEST_BODY_SIZE_LIMIT_BYTES: usize = 1_024;

    async fn raw_response_url(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 1024];
            let _ = socket.read(&mut request).await;
            socket.write_all(response).await.unwrap();
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
        let result = check_url(
            &client,
            server.base_url(),
            &check,
            1,
            TEST_BODY_SIZE_LIMIT_BYTES,
            false,
        )
        .await;
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
        let allowed = check_url(
            &client,
            server.base_url(),
            &check,
            1,
            TEST_BODY_SIZE_LIMIT_BYTES,
            true,
        )
        .await;
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
        let result = check_url(
            &blocked_client,
            &url,
            &check,
            1,
            TEST_BODY_SIZE_LIMIT_BYTES,
            false,
        )
        .await;
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
        let allowed = check_url(
            &allowed_client,
            &url,
            &check,
            1,
            TEST_BODY_SIZE_LIMIT_BYTES,
            true,
        )
        .await;
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
            let result = check_url(
                &client,
                server.base_url(),
                &check,
                1,
                TEST_BODY_SIZE_LIMIT_BYTES,
                true,
            )
            .await;
            assert_eq!(result.status, status);
            assert_eq!(result.error_message.as_deref(), error_message);
        }
    }

    #[tokio::test]
    async fn test_check_url_reports_response_body_read_failure() {
        let url = raw_response_url(
            b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\nConnection: close\r\n\r\nshort",
        )
        .await;
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: Some("healthy".to_owned()),
        };
        let result = check_url(
            &Client::new(),
            &url,
            &check,
            1,
            TEST_BODY_SIZE_LIMIT_BYTES,
            true,
        )
        .await;
        assert!(result.status.is_down());
        assert!(
            result
                .error_message
                .as_deref()
                .is_some_and(|message| message.starts_with(RESPONSE_BODY_READ_ERROR_PREFIX))
        );
    }

    #[tokio::test]
    async fn test_check_url_bounds_expected_text_response_body() {
        let chunked_url = raw_response_url(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\nheal\r\n3\r\nthy\r\n0\r\n\r\n",
        )
        .await;
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: Some("healthy".to_owned()),
        };
        let result = check_url(&Client::new(), &chunked_url, &check, 1, 7, true).await;
        assert!(result.status.is_up());

        let early_match_url = raw_response_url(
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nokmore",
        )
        .await;
        let early_match = CheckExpectation {
            expected_status: 200,
            expected_text: Some("ok".to_owned()),
        };
        let result = check_url(&Client::new(), &early_match_url, &early_match, 1, 2, true).await;
        assert!(result.status.is_up());
        let exact_limit_url = raw_response_url(
            b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nabcde",
        )
        .await;
        let result = check_url(&Client::new(), &exact_limit_url, &check, 1, 5, true).await;
        assert!(result.status.is_down());
        assert_eq!(
            result.error_message.as_deref(),
            Some(EXPECTED_TEXT_MISSING_MESSAGE)
        );
        let oversized_url = raw_response_url(
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nabcdef",
        )
        .await;
        let result = check_url(&Client::new(), &oversized_url, &check, 1, 5, true).await;
        assert!(result.status.is_down());
        assert_eq!(
            result.error_message.as_deref(),
            Some(
                "Response body exceeded configured limit of 5 bytes before expected text was found"
            )
        );
    }
}
