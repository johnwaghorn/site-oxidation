use crate::models::site::{CertStatus, SiteStatus};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct SiteResponse {
    pub id: i64,
    pub name: String,
    pub url: String,
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
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct OutageResponse {
    pub id: i64,
    pub site_id: i64,
    pub http_status: Option<i64>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}
