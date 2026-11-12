use crate::models::site::{CertStatus, SiteStatus};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct SiteResponse {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
    pub expected_status: i64,
    pub expected_text: Option<String>,
    pub status: SiteStatus,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_response_time_ms: Option<i64>,
    pub probe_interval_seconds: i64,
    pub team_id: Option<i64>,
    pub team_name: Option<String>,
    pub tls_allow_untrusted: bool,
    pub cert_status: Option<CertStatus>,
    pub cert_expires_at: Option<DateTime<Utc>>,
    #[sqlx(skip)]
    pub canary_warning: Option<CanaryWarning>,
    #[sqlx(skip)]
    pub canary_warning_since: Option<DateTime<Utc>>,
    /// Uptime over the last `uptime_window_days` days, or since the site was added if that is more recent.
    #[sqlx(skip)]
    pub uptime_basis_points: Option<i64>,
    #[sqlx(skip)]
    pub uptime_window_days: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanaryWarning {
    Inactive,
    SharesHost,
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct OutageResponse {
    pub id: i64,
    pub site_id: i64,
    pub http_status: Option<i64>,
    pub expected_status: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct SiteUptimeResponse {
    pub uptime_basis_points: Option<i64>,
    pub unwatched_seconds: i64,
    pub down_since: Option<DateTime<Utc>>,
    pub days: Vec<UptimeDay>,
}

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct UptimeDay {
    pub date: NaiveDate,
    pub monitoring_started: bool,
    pub uptime_basis_points: Option<i64>,
    pub watched_seconds: i64,
    pub down_seconds: i64,
    pub outage_count: i64,
    pub canary_healthy_seconds: i64,
    pub canary_shares_host_seconds: i64,
    pub canary_failing_seconds: i64,
    pub no_canary_seconds: i64,
    pub not_running_seconds: i64,
    pub watch_segments: Vec<WatchSegment>,
}

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct WatchSegment {
    pub state: WatchStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    CanaryHealthy,
    CanarySharesHost,
    NoCanary,
    CanaryFailing,
    NotRunning,
}

#[derive(Serialize, ToSchema)]
pub struct UnassignedCountResponse {
    pub count: i64,
}
