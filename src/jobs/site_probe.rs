use crate::config::AppConfig;
use crate::models::site::SiteRow;
use crate::probe::cert::{CertCheck, CertExpiryWindows, check_certificate};
use crate::probe::http::{CheckExpectation, ProbeResult, check_url};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sqlx::SqlitePool;
use std::time::Duration;

pub async fn check_all_sites(
    verifying_client: &Client,
    untrusted_client: &Client,
    pool: &SqlitePool,
    config: &AppConfig,
) {
    if verifying_client
        .head(&config.canary_url)
        .timeout(Duration::from_secs(config.canary_timeout_secs))
        .send()
        .await
        .is_err()
    {
        tracing::warn!("Canary check failed. Skipping sites check. Network issue?");
        return;
    }
    let sites = sqlx::query_as::<_, SiteRow>(
        r"
            SELECT id, name, url, expected_status, expected_text, status, probe_interval_seconds, tls_allow_untrusted
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
        .map(|site| check_single_site(verifying_client, untrusted_client, pool, config, site))
        .buffer_unordered(config.probe_max_concurrent_checks)
        .collect::<Vec<()>>()
        .await;
    tracing::info!("Finished checking {} sites", site_count);
}

async fn check_single_site(
    verifying_client: &Client,
    untrusted_client: &Client,
    pool: &SqlitePool,
    config: &AppConfig,
    site: SiteRow,
) {
    tracing::info!(
        "Checking site {} (interval: {}s)",
        site.name,
        site.probe_interval_seconds
    );
    let probe_client = if site.tls_allow_untrusted {
        untrusted_client
    } else {
        verifying_client
    };
    let check = CheckExpectation {
        expected_status: u16::try_from(site.expected_status).unwrap_or(200),
        expected_text: site.expected_text.clone(),
    };
    let mut probe_result = check_url(
        probe_client,
        &site.url,
        &check,
        config.probe_timeout_secs,
        config.probe_allow_private_ips,
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
            probe_result = check_url(
                probe_client,
                &site.url,
                &check,
                config.probe_timeout_secs,
                config.probe_allow_private_ips,
            )
            .await;
            if probe_result.status.is_up() {
                tracing::info!("Site '{}' recovered on retry {}", site.name, attempt);
                break;
            }
        }
    }
    update_site_status(pool, &site, &probe_result).await;
    if probe_result.status.is_blocked() {
        clear_site_cert(pool, site.id).await;
    } else {
        let cert = check_certificate(
            &site.url,
            site.tls_allow_untrusted,
            config.probe_allow_private_ips,
            Duration::from_secs(config.probe_timeout_secs),
            Utc::now(),
            CertExpiryWindows {
                warn_days: config.cert_warn_days,
                critical_days: config.cert_critical_days,
            },
        )
        .await;
        update_site_cert_status(pool, site.id, &cert).await;
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
                .map_or_else(|| "N/A".to_owned(), |c| c.to_string()),
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
    if site.status.is_down() && !result.status.is_down() {
        if result.status.is_blocked() {
            tracing::info!(
                "Site '{}' outage closed - probe is now blocked (see prior warning for reason)",
                site.name
            );
        } else {
            tracing::info!("Site '{}' is back UP", site.name);
        }
        sqlx::query("UPDATE outages SET ended_at = ? WHERE site_id = ? AND ended_at IS NULL")
            .bind(Utc::now())
            .bind(site.id)
            .execute(pool)
            .await
            .map_err(|e| tracing::error!("Failed to close outage for site {}: {}", site.id, e))
            .ok();
    }
}

pub async fn update_site_cert_status(pool: &SqlitePool, site_id: i64, cert: &CertCheck) {
    sqlx::query(
        "UPDATE sites SET cert_status = ?, cert_expires_at = ?, cert_checked_at = ? WHERE id = ?",
    )
    .bind(cert.status)
    .bind(cert.expires_at)
    .bind(Utc::now())
    .bind(site_id)
    .execute(pool)
    .await
    .map_err(|e| tracing::error!("Failed to update cert status for site {}: {}", site_id, e))
    .ok();
}

async fn clear_site_cert(pool: &SqlitePool, site_id: i64) {
    sqlx::query(
        "UPDATE sites SET cert_status = NULL, cert_expires_at = NULL, cert_checked_at = NULL WHERE id = ?",
    )
    .bind(site_id)
    .execute(pool)
    .await
    .map_err(|e| tracing::error!("Failed to clear cert status for site {}: {}", site_id, e))
    .ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::site::SiteStatus;
    use crate::tests::{insert_test_site, test_config};
    use reqwest::StatusCode;
    use tracing_test::traced_test;

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

    #[sqlx::test(migrations = "./migrations")]
    #[traced_test]
    async fn test_canary_failure_is_logged(pool: SqlitePool) {
        let client = Client::new();
        let mut config = test_config(true);
        config.canary_url = "not a url".to_owned();
        check_all_sites(&client, &client, &pool, &config).await;
        assert!(logs_contain(
            "Canary check failed. Skipping sites check. Network issue?"
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    #[traced_test]
    async fn test_outage_created_when_site_goes_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Up).await;
        update_site_status(&pool, &site, &mock_site_down_result()).await;
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
        assert!(logs_contain("Site 'Waghorn Technology Ltd' is DOWN"));
    }

    #[sqlx::test(migrations = "./migrations")]
    #[traced_test]
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
        assert!(logs_contain("Site 'Waghorn Technology Ltd' is back UP"));
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
