use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SiteExportResponse {
    pub version: u32,
    pub exported_at: DateTime<Utc>,
    pub exported_by: &'static str,
    pub sites: Vec<ExportedSite>,
}

#[derive(Serialize, sqlx::FromRow, ToSchema)]
pub struct ExportedSite {
    pub name: String,
    pub url: String,
    pub expected_status: i64,
    pub expected_text: Option<String>,
    pub probe_interval_seconds: i64,
    pub tls_allow_untrusted: bool,
}
