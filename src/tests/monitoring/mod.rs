use super::checks::{probe_result_is_stale, run_due_site_checks};
use super::connectivity::{
    AmbiguousFailureGuard, CANARY_RECHECK_CACHE_TTL, CachedCanaryVerdict, CanaryVerdict,
    run_and_record_canary,
};
use super::persistence::{persist_site_statuses, update_site_status};
use crate::config::AppConfig;
use crate::models::site::SiteStatus;
use crate::notifications::Notifier;
use crate::probe::http::{PRIVATE_IP_BLOCKED_MESSAGE, ProbeFailureKind, ProbeResult};
use crate::tests::{
    TestHttpServer, count_outages, enable_canary, insert_team, insert_test_site, test_config,
};
use reqwest::{Client, StatusCode};
use sqlx::SqlitePool;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use tracing_test::traced_test;

mod checks;
mod connectivity;
mod persistence;

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
    let team_id = insert_team(pool, &format!("{name} Team")).await;
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

fn probe_config() -> AppConfig {
    let mut config = test_config(true);
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
        failure_kind: Some(ProbeFailureKind::ResponseRejected),
    }
}

fn mock_site_up_result() -> ProbeResult {
    ProbeResult {
        status: SiteStatus::Up,
        status_code: Some(StatusCode::OK),
        latency_ms: Some(100),
        error_message: None,
        failure_kind: None,
    }
}

fn mock_site_blocked_result() -> ProbeResult {
    ProbeResult {
        status: SiteStatus::Blocked,
        status_code: None,
        latency_ms: None,
        error_message: Some(PRIVATE_IP_BLOCKED_MESSAGE.to_owned()),
        failure_kind: None,
    }
}
