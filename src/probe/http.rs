use crate::models::site::SiteStatus;
use crate::security::resolver::{ResolveError, resolve_public_addrs};
use reqwest::{Client, StatusCode};
use std::time::Duration;

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
        if let Err(ResolveError::PrivateIp { .. }) = resolve_public_addrs(host, port, false).await {
            return ProbeResult {
                status: SiteStatus::Blocked,
                status_code: None,
                latency_ms: None,
                error_message: None,
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
        Ok(res) => {
            let status_code = res.status();
            let latency_ms = start.elapsed().as_millis();
            let status_matches = status_code.as_u16() == check.expected_status;
            let text_matches = if let Some(expected_text) = &check.expected_text {
                match res.text().await {
                    Ok(body) => body.contains(expected_text),
                    Err(_) => false,
                }
            } else {
                true
            };
            let is_up = status_matches && text_matches;
            ProbeResult {
                status: if is_up {
                    SiteStatus::Up
                } else {
                    SiteStatus::Down
                },
                status_code: Some(status_code),
                latency_ms: Some(latency_ms),
                error_message: None,
            }
        }
        Err(e) => ProbeResult {
            status: SiteStatus::Down,
            status_code: None,
            latency_ms: None,
            error_message: Some(e.to_string().chars().take(500).collect()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::resolver::SafeResolver;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    async fn start_local_http_server() -> (u16, JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 2048];
                let _ = socket.read(&mut buf).await;
                let response =
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                let _ = socket.write_all(response).await;
            }
        });
        (port, handle)
    }

    #[tokio::test]
    async fn test_check_url_blocks_literal_private_ip_when_private_ips_disabled() {
        let client = Client::new();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let result = check_url(&client, "http://127.0.0.1:1", &check, 1, false).await;
        assert!(
            result.status.is_blocked(),
            "literal private IP should mark site as Blocked"
        );
    }

    #[tokio::test]
    async fn test_check_url_allows_literal_private_ip_when_private_ips_enabled() {
        let (port, server_handle) = start_local_http_server().await;
        let client = Client::new();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let url = format!("http://127.0.0.1:{port}");
        let result = check_url(&client, &url, &check, 1, true).await;
        assert!(result.status.is_up());
        assert_eq!(result.status_code, Some(StatusCode::OK));
        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_check_url_blocks_hostname_that_resolves_to_private_ip() {
        let (port, server_handle) = start_local_http_server().await;
        let client = Client::builder()
            .dns_resolver(Arc::new(SafeResolver {
                allow_private: false,
            }))
            .build()
            .unwrap();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let url = format!("http://localhost:{port}");
        let result = check_url(&client, &url, &check, 1, false).await;
        assert!(
            result.status.is_blocked(),
            "hostname resolving to a private IP should mark site as Blocked"
        );
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_check_url_allows_hostname_resolving_to_private_ip_when_private_ips_enabled() {
        let (port, _server_handle) = start_local_http_server().await;
        let client = Client::builder()
            .dns_resolver(Arc::new(SafeResolver {
                allow_private: true,
            }))
            .build()
            .unwrap();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let url = format!("http://localhost:{port}");
        let result = check_url(&client, &url, &check, 1, true).await;
        assert!(result.status.is_up());
        assert_eq!(result.status_code, Some(StatusCode::OK));
        _server_handle.await.unwrap();
    }
}
