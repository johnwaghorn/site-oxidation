use crate::config::{AppConfig, CANARY_TIMEOUT_SECS, CANARY_URL, PROBE_MAX_CONCURRENT_CHECKS};
use crate::models::{SiteRow, SiteStatus};
use crate::net::is_private_ip;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::{Client, StatusCode};
use sqlx::SqlitePool;
use std::net::IpAddr;
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

pub async fn check_all_sites(client: &Client, pool: &SqlitePool, config: &AppConfig) {
    if client
        .head(CANARY_URL)
        .timeout(Duration::from_secs(CANARY_TIMEOUT_SECS))
        .send()
        .await
        .is_err()
    {
        tracing::warn!("Canary check failed. Skipping sites check. Network issue?");
        return;
    }
    let sites = sqlx::query_as::<_, SiteRow>(
        r"
            SELECT id, name, url, expected_status, expected_text, status, probe_interval_seconds
            FROM sites
            WHERE last_checked_at IS NULL
                OR datetime(last_checked_at, '+' || COALESCE(probe_interval_seconds, 60) || ' seconds') <= datetime('now')
            ",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if sites.is_empty() {
        tracing::info!("No sites due for a probe");
        return;
    }
    let site_count = sites.len();
    stream::iter(sites)
        .map(|site| check_single_site(client, pool, config, site))
        .buffer_unordered(PROBE_MAX_CONCURRENT_CHECKS)
        .collect::<Vec<()>>()
        .await;
    tracing::info!("Finished checking {} sites", site_count);
}

async fn check_single_site(client: &Client, pool: &SqlitePool, config: &AppConfig, site: SiteRow) {
    tracing::info!(
        "Checking site {} (interval: {}s)",
        site.name,
        site.probe_interval_seconds
    );
    let check = CheckExpectation {
        expected_status: u16::try_from(site.expected_status).unwrap_or(200),
        expected_text: site.expected_text.clone(),
    };
    let mut probe_result = probe_site(
        client,
        &site.url,
        &check,
        config.probe_timeout_secs,
        config.allow_private_ips,
    )
    .await;
    if probe_result.status.is_down() && !site.status.is_down() {
        for attempt in 1..=config.probe_retry_count {
            tracing::info!(
                "Site '{}' probe failed, retry {}/{} after {}ms",
                site.name,
                attempt,
                config.probe_retry_count,
                config.probe_retry_delay_ms
            );
            tokio::time::sleep(Duration::from_millis(config.probe_retry_delay_ms)).await;
            probe_result = probe_site(
                client,
                &site.url,
                &check,
                config.probe_timeout_secs,
                config.allow_private_ips,
            )
            .await;
            if probe_result.status.is_up() {
                tracing::info!("Site '{}' recovered on retry {}", site.name, attempt);
                break;
            }
        }
    }
    update_site_status(pool, &site, &probe_result).await;
}

pub async fn probe_site(
    client: &Client,
    url: &str,
    check: &CheckExpectation,
    timeout_secs: u64,
    allow_private_ips: bool,
) -> ProbeResult {
    if !allow_private_ips && let Ok(parsed) = reqwest::Url::parse(url) {
        let is_private = match parsed.host() {
            Some(url::Host::Ipv4(ip)) => is_private_ip(&IpAddr::V4(ip)),
            Some(url::Host::Ipv6(ip)) => is_private_ip(&IpAddr::V6(ip)),
            _ => false,
        };
        if is_private {
            return ProbeResult {
                status: SiteStatus::Down,
                status_code: None,
                latency_ms: None,
                error_message: Some("blocked: URL host is private IP literal".to_string()),
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

pub async fn update_site_status(pool: &SqlitePool, site: &SiteRow, result: &ProbeResult) {
    sqlx::query(
        "UPDATE sites SET status = ?, last_checked_at = ?, last_response_time_ms = ? WHERE id = ?",
    )
    .bind(result.status)
    .bind(Utc::now())
    .bind(
        result
            .latency_ms
            .map(|ms| i64::try_from(ms).unwrap_or(i64::MAX)),
    )
    .bind(site.id)
    .execute(pool)
    .await
    .map_err(|e| tracing::error!("Failed to update site status for site {}: {}", site.id, e))
    .ok();
    if !site.status.is_down() && result.status.is_down() {
        tracing::warn!(
            "Site '{}' is DOWN (status: {}) - {}",
            site.name,
            result
                .status_code
                .map_or_else(|| "N/A".to_string(), |c| c.to_string()),
            result
                .error_message
                .as_deref()
                .unwrap_or("no error message")
        );
        sqlx::query("INSERT INTO outages (site_id, http_status, error_message) VALUES (?, ?, ?)")
            .bind(site.id)
            .bind(result.status_code.map(|c| i64::from(c.as_u16())))
            .bind(&result.error_message)
            .execute(pool)
            .await
            .map_err(|e| tracing::error!("Failed to insert outage for site {}: {}", site.id, e))
            .ok();
    }
    if site.status.is_down() && result.status.is_up() {
        tracing::info!("Site '{}' is back UP", site.name);
        sqlx::query("UPDATE outages SET ended_at = ? WHERE site_id = ? AND ended_at IS NULL")
            .bind(Utc::now())
            .bind(site.id)
            .execute(pool)
            .await
            .map_err(|e| tracing::error!("Failed to close outage for site {}: {}", site.id, e))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::SafeResolver;
    use crate::tests::insert_test_site;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::task::JoinHandle;

    fn mock_site_down_result() -> ProbeResult {
        ProbeResult {
            status: SiteStatus::Down,
            status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            latency_ms: Some(500),
            error_message: Some(String::from("Server is cooked")),
        }
    }

    fn mock_site_up_result() -> ProbeResult {
        ProbeResult {
            status: SiteStatus::Up,
            status_code: Some(StatusCode::OK),
            latency_ms: Some(100),
            error_message: None,
        }
    }

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
    async fn test_probe_site_blocks_literal_private_ip_when_private_ips_disabled() {
        let client = Client::new();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let result = probe_site(&client, "http://127.0.0.1:1", &check, 1, false).await;
        assert!(result.status.is_down());
        assert_eq!(
            result.error_message.as_deref(),
            Some("blocked: URL host is private IP literal")
        );
    }

    #[tokio::test]
    async fn test_probe_site_allows_literal_private_ip_when_private_ips_enabled() {
        let (port, server_handle) = start_local_http_server().await;
        let client = Client::new();
        let check = CheckExpectation {
            expected_status: 200,
            expected_text: None,
        };
        let url = format!("http://127.0.0.1:{port}");
        let result = probe_site(&client, &url, &check, 1, true).await;
        assert!(result.status.is_up());
        assert_eq!(result.status_code, Some(StatusCode::OK));
        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_probe_site_blocks_hostname_that_resolves_to_private_ip() {
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
        let result = probe_site(&client, &url, &check, 1, false).await;
        assert!(result.status.is_down());
        assert!(
            result.error_message.is_some(),
            "expected request failure, got no error message"
        );
        assert_ne!(
            result.error_message.as_deref(),
            Some("blocked: URL host is private IP literal")
        );
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_probe_site_allows_hostname_resolving_to_private_ip_when_private_ips_enabled() {
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
        let result = probe_site(&client, &url, &check, 1, true).await;
        assert!(result.status.is_up());
        assert_eq!(result.status_code, Some(StatusCode::OK));
        _server_handle.await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_created_when_site_goes_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Up).await;
        update_site_status(&pool, &site, &mock_site_down_result()).await;
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_closed_when_site_recovers(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Down).await;
        sqlx::query("INSERT INTO outages (site_id, http_status, error_message) VALUES (?, ?, ?)")
            .bind(site.id)
            .bind(500)
            .bind(String::from("Server cooked"))
            .execute(&pool)
            .await
            .unwrap();
        update_site_status(&pool, &site, &mock_site_up_result()).await;
        let outage_ended: Option<String> =
            sqlx::query_scalar("SELECT ended_at FROM outages WHERE site_id = ?")
                .bind(site.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(outage_ended.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_no_duplicate_outage_when_already_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Down).await;
        update_site_status(&pool, &site, &mock_site_down_result()).await;
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_created_when_pending_site_goes_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Pending).await;
        update_site_status(&pool, &site, &mock_site_down_result()).await;
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }
}
