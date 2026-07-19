use crate::config::AppConfig;
use crate::models::site::{CertStatus, SiteRow};
use crate::notifications::{Notifier, planning};
use crate::probe::cert::{CertCheck, CertExpiryWindows, check_certificate};
use crate::probe::http::{CheckExpectation, ProbeResult, check_url};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::collections::HashMap;
use std::time::Duration;

pub enum SiteTransition {
    WentDown,
    Recovered,
    NoChange,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct ProbeGroupKey {
    url: String,
    expected_status: i64,
    expected_text: Option<String>,
    tls_allow_untrusted: bool,
}

impl From<&SiteRow> for ProbeGroupKey {
    fn from(site: &SiteRow) -> Self {
        Self {
            url: site.url.clone(),
            expected_status: site.expected_status,
            expected_text: site.expected_text.clone(),
            tls_allow_untrusted: site.tls_allow_untrusted,
        }
    }
}

pub async fn check_all_sites(
    verifying_client: &Client,
    untrusted_client: &Client,
    pool: &SqlitePool,
    config: &AppConfig,
    notifier: &Notifier,
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
    let sites = match sqlx::query_as::<_, SiteRow>(
        r"
            SELECT s.id, s.name, s.url, s.expected_status, s.expected_text, s.status,
                   s.tls_allow_untrusted, s.cert_status, n.slack_webhook_url,
                   n.microsoft_teams_webhook_url,
                   n.smtp_host, n.smtp_port,
                   COALESCE(n.smtp_tls_mode, 'starttls') AS smtp_tls_mode,
                   COALESCE(n.smtp_auth, 1) AS smtp_auth,
                   n.smtp_username, n.smtp_password, n.smtp_from_email, n.smtp_to_email,
                   COALESCE(n.notify_site_down, 1) AS notify_site_down,
                   COALESCE(n.notify_site_recovered, 1) AS notify_site_recovered,
                   COALESCE(n.notify_cert_expiring, 1) AS notify_cert_expiring
            FROM sites s
            LEFT JOIN team_notification_settings n ON n.team_id = s.team_id
            WHERE EXISTS (
                SELECT 1
                FROM sites due
                WHERE due.url = s.url
                    AND due.expected_status = s.expected_status
                    AND due.expected_text IS s.expected_text
                    AND due.tls_allow_untrusted = s.tls_allow_untrusted
                    AND due.probe_interval_seconds = s.probe_interval_seconds
                    AND (
                        due.last_checked_at IS NULL
                        OR datetime(due.last_checked_at, '+' || COALESCE(due.probe_interval_seconds, 60) || ' seconds') <= datetime('now')
                    )
            )
            ",
    )
    .fetch_all(pool)
    .await
    {
        Ok(sites) => sites,
        Err(error) => {
            tracing::error!("Failed to load sites due for probing: {error}");
            return;
        }
    };
    if sites.is_empty() {
        tracing::info!("No sites due for a probe");
        return;
    }
    let site_count = sites.len();
    let mut grouped_sites: HashMap<ProbeGroupKey, Vec<SiteRow>> = HashMap::new();
    for site in sites {
        grouped_sites
            .entry(ProbeGroupKey::from(&site))
            .or_default()
            .push(site);
    }
    let probe_count = grouped_sites.len();
    stream::iter(grouped_sites)
        .map(|(group_key, group_sites)| {
            check_site_group(
                verifying_client,
                untrusted_client,
                pool,
                config,
                notifier,
                group_key,
                group_sites,
            )
        })
        .buffer_unordered(config.probe_max_concurrent_checks)
        .collect::<Vec<()>>()
        .await;
    notifier.process_outbox(pool).await;
    tracing::info!(
        "Finished checking {} sites in {} probes",
        site_count,
        probe_count
    );
}

async fn check_site_group(
    verifying_client: &Client,
    untrusted_client: &Client,
    pool: &SqlitePool,
    config: &AppConfig,
    notifier: &Notifier,
    group_key: ProbeGroupKey,
    group_sites: Vec<SiteRow>,
) {
    tracing::info!(
        "Checking URL '{}' for {} monitor(s)",
        group_key.url,
        group_sites.len()
    );
    let probe_client = if group_key.tls_allow_untrusted {
        untrusted_client
    } else {
        verifying_client
    };
    let check = CheckExpectation {
        expected_status: u16::try_from(group_key.expected_status).unwrap_or(200),
        expected_text: group_key.expected_text.clone(),
    };
    let mut probe_result = check_url(
        probe_client,
        &group_key.url,
        &check,
        config.probe_timeout_secs,
        config.probe_allow_private_ips,
    )
    .await;
    if probe_result.status.is_down() && group_sites.iter().any(|site| !site.status.is_down()) {
        for attempt in 1..=config.probe_retry_count {
            tracing::info!(
                "URL '{}' probe failed, retry {}/{} after {}ms",
                group_key.url,
                attempt,
                config.probe_retry_count,
                config.probe_retry_delay_ms
            );
            tokio::time::sleep(Duration::from_millis(config.probe_retry_delay_ms)).await;
            probe_result = check_url(
                probe_client,
                &group_key.url,
                &check,
                config.probe_timeout_secs,
                config.probe_allow_private_ips,
            )
            .await;
            if probe_result.status.is_up() {
                tracing::info!("URL '{}' recovered on retry {}", group_key.url, attempt);
                break;
            }
        }
    }
    if let Err(error) = persist_site_statuses(pool, &group_sites, &probe_result, notifier).await {
        tracing::error!(
            "Failed to persist probe result for '{}': {error}",
            group_key.url
        );
        return;
    }
    if probe_result.status.is_blocked() {
        for site in &group_sites {
            clear_site_cert(pool, site.id).await;
        }
    } else {
        let cert = check_certificate(
            &group_key.url,
            group_key.tls_allow_untrusted,
            config.probe_allow_private_ips,
            Duration::from_secs(config.probe_timeout_secs),
            Utc::now(),
            CertExpiryWindows {
                warn_days: config.cert_warn_days,
                critical_days: config.cert_critical_days,
            },
        )
        .await;
        if let Err(error) = persist_site_cert_results(pool, &group_sites, &cert, notifier).await {
            tracing::error!(
                "Failed to persist certificate result for '{}': {error}",
                group_key.url
            );
        }
    }
}

async fn persist_site_statuses(
    pool: &SqlitePool,
    sites: &[SiteRow],
    result: &ProbeResult,
    notifier: &Notifier,
) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;
    let mut went_down = Vec::new();
    let mut recovered = Vec::new();
    let mut blocked = Vec::new();
    for site in sites {
        match update_site_status(&mut transaction, site, result).await? {
            SiteTransition::WentDown => went_down.push(site),
            SiteTransition::Recovered => recovered.push(site),
            SiteTransition::NoChange => {
                if site.status.is_down() && result.status.is_blocked() {
                    blocked.push(site);
                }
            }
        }
    }
    let mut deliveries = planning::site_down(&went_down, result)?;
    deliveries.extend(planning::site_recovered(&recovered)?);
    notifier.enqueue(&mut transaction, &deliveries).await?;
    transaction.commit().await?;
    for site in went_down {
        tracing::warn!(
            "Site '{}' is DOWN (status: {}) - {}",
            site.name,
            result
                .status_code
                .map_or_else(|| "N/A".to_owned(), |code| code.to_string()),
            result
                .error_message
                .as_deref()
                .unwrap_or("no error message")
        );
    }
    for site in recovered {
        tracing::info!("Site '{}' is back UP", site.name);
    }
    for site in blocked {
        tracing::info!(
            "Site '{}' outage closed - probe is now blocked (see prior warning for reason)",
            site.name
        );
    }
    Ok(())
}

async fn persist_site_cert_results(
    pool: &SqlitePool,
    sites: &[SiteRow],
    cert: &CertCheck,
    notifier: &Notifier,
) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;
    let newly_expiring: Vec<&SiteRow> = sites
        .iter()
        .filter(|site| cert_newly_expiring(site, cert))
        .collect();
    for site in sites {
        update_site_cert_status(&mut transaction, site.id, cert).await?;
    }
    let deliveries = planning::cert_expiring(&newly_expiring, cert)?;
    notifier.enqueue(&mut transaction, &deliveries).await?;
    transaction.commit().await?;
    Ok(())
}

fn cert_newly_expiring(site: &SiteRow, cert: &CertCheck) -> bool {
    matches!(
        cert.status,
        CertStatus::Expiring | CertStatus::Critical | CertStatus::Expired
    ) && site.cert_status != Some(cert.status)
}

pub async fn update_site_status(
    transaction: &mut Transaction<'_, Sqlite>,
    site: &SiteRow,
    result: &ProbeResult,
) -> sqlx::Result<SiteTransition> {
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
    .execute(&mut **transaction)
    .await?;
    if !site.status.is_down() && result.status.is_down() {
        sqlx::query(
            "INSERT INTO outages (site_id, http_status, error_message, expected_status) VALUES (?, ?, ?, ?)",
        )
            .bind(site.id)
            .bind(result.status_code.map(|c| i64::from(c.as_u16())))
            .bind(&result.error_message)
            .bind(site.expected_status)
            .execute(&mut **transaction)
            .await?;
        return Ok(SiteTransition::WentDown);
    }
    if site.status.is_down() && !result.status.is_down() {
        sqlx::query("UPDATE outages SET ended_at = ? WHERE site_id = ? AND ended_at IS NULL")
            .bind(Utc::now())
            .bind(site.id)
            .execute(&mut **transaction)
            .await?;
        if result.status.is_up() {
            return Ok(SiteTransition::Recovered);
        }
    }
    Ok(SiteTransition::NoChange)
}

async fn update_site_cert_status(
    transaction: &mut Transaction<'_, Sqlite>,
    site_id: i64,
    cert: &CertCheck,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE sites SET cert_status = ?, cert_expires_at = ?, cert_checked_at = ? WHERE id = ?",
    )
    .bind(cert.status)
    .bind(cert.expires_at)
    .bind(Utc::now())
    .bind(site_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
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
    use crate::tests::{TestHttpServer, insert_test_site, test_config};
    use reqwest::StatusCode;
    use tracing_test::traced_test;

    async fn insert_probe_site(
        pool: &SqlitePool,
        name: &str,
        url: &str,
        expected_status: i64,
        expected_text: Option<&str>,
        status: SiteStatus,
        probe_interval_seconds: i64,
        tls_allow_untrusted: bool,
    ) -> i64 {
        let team_id: i64 = sqlx::query_scalar("INSERT INTO teams (name) VALUES (?) RETURNING id")
            .bind(format!("{name} Team"))
            .fetch_one(pool)
            .await
            .unwrap();
        sqlx::query_scalar(
            "INSERT INTO sites (
                name, url, expected_status, expected_text, status,
                probe_interval_seconds, tls_allow_untrusted, team_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        )
        .bind(name)
        .bind(url)
        .bind(expected_status)
        .bind(expected_text)
        .bind(status)
        .bind(probe_interval_seconds)
        .bind(tls_allow_untrusted)
        .bind(team_id)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    fn probe_config(base_url: &str) -> AppConfig {
        let mut config = test_config(true);
        config.canary_url = format!("{base_url}/canary");
        config.probe_retry_count = 0;
        config.probe_retry_delay_ms = 0;
        config
    }

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
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert!(logs_contain(
            "Canary check failed. Skipping sites check. Network issue?"
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    #[traced_test]
    async fn test_equivalent_due_sites_with_different_intervals_share_probe(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(
            &pool,
            "Site A",
            &url,
            200,
            None,
            SiteStatus::Pending,
            60,
            false,
        )
        .await;
        insert_probe_site(
            &pool,
            "Site B",
            &url,
            200,
            None,
            SiteStatus::Pending,
            300,
            false,
        )
        .await;
        let client = Client::new();
        let config = probe_config(base_url);
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 1);
        let updated: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sites WHERE status = 'up' AND last_checked_at IS NOT NULL",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(updated, 2);
        assert!(logs_contain("Finished checking 2 sites in 1 probes"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_due_site_coalesces_recent_equivalent_monitor_with_same_interval(
        pool: SqlitePool,
    ) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(
            &pool,
            "Due",
            &url,
            200,
            None,
            SiteStatus::Pending,
            60,
            false,
        )
        .await;
        let recent_id = insert_probe_site(
            &pool,
            "Recent",
            &url,
            200,
            None,
            SiteStatus::Pending,
            60,
            false,
        )
        .await;
        sqlx::query("UPDATE sites SET last_checked_at = datetime('now') WHERE id = ?")
            .bind(recent_id)
            .execute(&pool)
            .await
            .unwrap();
        let client = Client::new();
        let config = probe_config(base_url);
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 1);
        let updated: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sites WHERE status = 'up' AND last_checked_at IS NOT NULL",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(updated, 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_shared_failure_creates_separate_outages(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(
            &pool,
            "Site A",
            &url,
            503,
            None,
            SiteStatus::Pending,
            60,
            false,
        )
        .await;
        insert_probe_site(
            &pool,
            "Site B",
            &url,
            503,
            None,
            SiteStatus::Pending,
            60,
            false,
        )
        .await;
        let client = Client::new();
        let config = probe_config(base_url);
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 1);
        let outages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(outages, 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_shared_failure_retries_when_any_site_was_not_down(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(&pool, "Site A", &url, 503, None, SiteStatus::Up, 60, false).await;
        insert_probe_site(
            &pool,
            "Site B",
            &url,
            503,
            None,
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        let client = Client::new();
        let mut config = probe_config(base_url);
        config.probe_retry_count = 1;
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 2);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_shared_failure_does_not_retry_when_all_sites_were_down(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(
            &pool,
            "Site A",
            &url,
            503,
            None,
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        insert_probe_site(
            &pool,
            "Site B",
            &url,
            503,
            None,
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        let client = Client::new();
        let mut config = probe_config(base_url);
        config.probe_retry_count = 1;
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_distinct_probe_keys_do_not_share_probe(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let base_url = server.base_url();
        let url = format!("{base_url}/site");
        insert_probe_site(&pool, "Base", &url, 200, None, SiteStatus::Down, 60, false).await;
        insert_probe_site(
            &pool,
            "Status",
            &url,
            201,
            None,
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        insert_probe_site(
            &pool,
            "Text",
            &url,
            200,
            Some("ok"),
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        insert_probe_site(&pool, "TLS", &url, 200, None, SiteStatus::Down, 60, true).await;
        insert_probe_site(
            &pool,
            "Trailing Slash",
            &format!("{base_url}/site/"),
            200,
            None,
            SiteStatus::Down,
            60,
            false,
        )
        .await;
        let client = Client::new();
        let config = probe_config(base_url);
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 5);
    }

    #[sqlx::test(migrations = "./migrations")]
    #[traced_test]
    async fn test_outage_created_when_site_goes_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Up).await;
        let config = test_config(true);
        persist_site_statuses(
            &pool,
            std::slice::from_ref(&site),
            &mock_site_down_result(),
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await
        .unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
        let expected_status: Option<i64> =
            sqlx::query_scalar("SELECT expected_status FROM outages WHERE site_id = ?")
                .bind(site.id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(expected_status, Some(site.expected_status));
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
        let config = test_config(true);
        persist_site_statuses(
            &pool,
            std::slice::from_ref(&site),
            &mock_site_up_result(),
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await
        .unwrap();
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
        let config = test_config(true);
        persist_site_statuses(
            &pool,
            std::slice::from_ref(&site),
            &mock_site_down_result(),
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await
        .unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_status_update_rolls_back_when_outage_insert_fails(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Up).await;
        sqlx::query("INSERT INTO outages (site_id) VALUES (?)")
            .bind(site.id)
            .execute(&pool)
            .await
            .unwrap();
        let mut transaction = pool.begin().await.unwrap();
        let result = update_site_status(&mut transaction, &site, &mock_site_down_result()).await;
        assert!(result.is_err());
        transaction.rollback().await.unwrap();
        let status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, SiteStatus::Up);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_created_when_pending_site_goes_down(pool: SqlitePool) {
        let site = insert_test_site(&pool, SiteStatus::Pending).await;
        let config = test_config(true);
        persist_site_statuses(
            &pool,
            std::slice::from_ref(&site),
            &mock_site_down_result(),
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await
        .unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_grouped_monitors_share_one_notification_per_webhook(pool: SqlitePool) {
        let server = TestHttpServer::start_ignoring_path("/canary").await;
        let dead_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let url = format!("http://127.0.0.1:{dead_port}/");
        for (team_name, monitor_name) in [("Team Rocket", "Monitor A"), ("Team Aqua", "Monitor B")]
        {
            let team_id: i64 =
                sqlx::query_scalar("INSERT INTO teams (name) VALUES (?) RETURNING id")
                    .bind(team_name)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            sqlx::query(
                "INSERT INTO team_notification_settings (team_id, slack_webhook_url) VALUES (?, ?)",
            )
            .bind(team_id)
            .bind(format!("{}/webhook", server.base_url()))
            .execute(&pool)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO sites (
                    name, url, expected_status, status,
                    probe_interval_seconds, tls_allow_untrusted, team_id
                ) VALUES (?, ?, 200, 'up', 60, 0, ?)",
            )
            .bind(monitor_name)
            .bind(&url)
            .bind(team_id)
            .execute(&pool)
            .await
            .unwrap();
        }
        let client = Client::new();
        let config = probe_config(server.base_url());
        check_all_sites(
            &client,
            &client,
            &pool,
            &config,
            &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
        )
        .await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("POST /webhook"));
        assert!(request.contains("is DOWN"));
    }
}
