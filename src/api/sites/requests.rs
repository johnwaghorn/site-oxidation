use super::fields::{CheckInterval, ExpectedStatus, ExpectedText, SiteName, SiteUrl};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct SitePayload {
    pub name: SiteName,
    pub url: SiteUrl,
    #[serde(default)]
    pub expected_status: ExpectedStatus,
    pub expected_text: Option<ExpectedText>,
    #[serde(default)]
    pub probe_interval_seconds: CheckInterval,
    pub team_id: Option<i64>,
    #[serde(default)]
    pub tls_allow_untrusted: bool,
}
