use super::connectivity::{AmbiguousFailureGuard, CanaryVerdict, run_and_record_canary};
use super::persistence::{clear_site_cert, persist_site_cert_results, persist_site_statuses};
use crate::config::AppConfig;
use crate::models::site::SiteRow;
use crate::notifications::Notifier;
use crate::probe::cert::{CertExpiryWindows, check_certificate};
use crate::probe::http::{CheckExpectation, ProbeResult, check_url};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;

#[derive(Clone, PartialEq, Eq, Hash)]
struct ProbeGroupKey {
    url: String,
    expected_status: i64,
    expected_text: Option<String>,
    tls_allow_untrusted: bool,
}

#[derive(Clone, Copy)]
struct ProbeClients<'a> {
    verifying: &'a Client,
    untrusted: &'a Client,
}

impl<'a> ProbeClients<'a> {
    fn for_group(self, group: &ProbeGroupKey) -> &'a Client {
        if group.tls_allow_untrusted {
            self.untrusted
        } else {
            self.verifying
        }
    }
}

struct CompletedProbe {
    result: ProbeResult,
    started_at: SystemTime,
    expected_duration: Duration,
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

pub(super) fn probe_result_is_stale(
    started_at: SystemTime,
    completed_at: SystemTime,
    expected_duration: Duration,
) -> bool {
    let max_age = expected_duration.saturating_add(Duration::from_secs(5));
    match completed_at.duration_since(started_at) {
        Ok(age) => age > max_age,
        Err(_) => true,
    }
}

pub async fn run_due_site_checks(
    verifying_client: &Client,
    untrusted_client: &Client,
    pool: &SqlitePool,
    config: &AppConfig,
    notifier: &Notifier,
) {
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
    let initial_canary_verdict = run_and_record_canary(verifying_client, pool).await;
    if initial_canary_verdict == CanaryVerdict::Inconclusive {
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
    let clients = ProbeClients {
        verifying: verifying_client,
        untrusted: untrusted_client,
    };
    let cached_recheck = Mutex::new(None);
    stream::iter(grouped_sites)
        .map(|(group_key, group_sites)| {
            check_site_group(
                clients,
                pool,
                config,
                notifier,
                AmbiguousFailureGuard {
                    initial_canary_verdict,
                    cached_recheck: &cached_recheck,
                },
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

async fn run_http_probe(
    client: &Client,
    config: &AppConfig,
    group: &ProbeGroupKey,
    sites: &[SiteRow],
) -> CompletedProbe {
    let check = CheckExpectation {
        expected_status: u16::try_from(group.expected_status).unwrap_or(200),
        expected_text: group.expected_text.clone(),
    };
    let probe_timeout = Duration::from_secs(config.probe_timeout_secs);
    let retry_delay = Duration::from_millis(config.probe_retry_delay_ms);
    let mut expected_duration = probe_timeout;
    let started_at = SystemTime::now();
    let mut result = check_url(
        client,
        &group.url,
        &check,
        config.probe_timeout_secs,
        config.probe_body_size_limit_bytes,
        config.probe_allow_private_ips,
    )
    .await;
    if result.status.is_down() && sites.iter().any(|site| !site.status.is_down()) {
        for attempt in 1..=config.probe_retry_count {
            tracing::info!(
                "URL '{}' probe failed, retry {}/{} after {}ms",
                group.url,
                attempt,
                config.probe_retry_count,
                config.probe_retry_delay_ms
            );
            tokio::time::sleep(retry_delay).await;
            expected_duration = expected_duration
                .saturating_add(retry_delay)
                .saturating_add(probe_timeout);
            result = check_url(
                client,
                &group.url,
                &check,
                config.probe_timeout_secs,
                config.probe_body_size_limit_bytes,
                config.probe_allow_private_ips,
            )
            .await;
            if result.status.is_up() {
                tracing::info!("URL '{}' recovered on retry {}", group.url, attempt);
                break;
            }
        }
    }
    CompletedProbe {
        result,
        started_at,
        expected_duration,
    }
}

async fn check_site_group(
    clients: ProbeClients<'_>,
    pool: &SqlitePool,
    config: &AppConfig,
    notifier: &Notifier,
    ambiguous_failure_guard: AmbiguousFailureGuard<'_>,
    group_key: ProbeGroupKey,
    group_sites: Vec<SiteRow>,
) {
    tracing::info!(
        "Checking URL '{}' for {} monitor(s)",
        group_key.url,
        group_sites.len()
    );
    let probe_timeout = Duration::from_secs(config.probe_timeout_secs);
    let completed = run_http_probe(
        clients.for_group(&group_key),
        config,
        &group_key,
        &group_sites,
    )
    .await;
    if completed.result.is_connectivity_ambiguous()
        && ambiguous_failure_guard
            .should_suppress_ambiguous_failure(clients.verifying, pool)
            .await
    {
        tracing::warn!(
            "Suppressing unreachable probe result for '{}' while probe connectivity is degraded",
            group_key.url
        );
        return;
    }
    let now_after_possible_recheck = SystemTime::now();
    if probe_result_is_stale(
        completed.started_at,
        now_after_possible_recheck,
        completed.expected_duration,
    ) {
        tracing::warn!("Discarding stale probe result for '{}'", group_key.url);
        return;
    }
    if let Err(error) = persist_site_statuses(pool, &group_sites, &completed.result, notifier).await
    {
        tracing::error!(
            "Failed to persist probe result for '{}': {error}",
            group_key.url
        );
        return;
    }
    if completed.result.status.is_blocked() {
        for site in &group_sites {
            clear_site_cert(pool, site.id).await;
        }
        return;
    }
    let cert_started_at = SystemTime::now();
    let cert = check_certificate(
        &group_key.url,
        group_key.tls_allow_untrusted,
        config.probe_allow_private_ips,
        probe_timeout,
        Utc::now(),
        CertExpiryWindows {
            warn_days: config.cert_warn_days,
            critical_days: config.cert_critical_days,
        },
    )
    .await;
    if probe_result_is_stale(cert_started_at, SystemTime::now(), probe_timeout) {
        tracing::warn!(
            "Discarding stale certificate result for '{}'",
            group_key.url
        );
        return;
    }
    if let Err(error) = persist_site_cert_results(pool, &group_sites, &cert, notifier).await {
        tracing::error!(
            "Failed to persist certificate result for '{}': {error}",
            group_key.url
        );
    }
}
