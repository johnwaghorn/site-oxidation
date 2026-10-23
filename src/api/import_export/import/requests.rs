use crate::api::sites::fields::{CheckInterval, ExpectedStatus, ExpectedText, SiteName, SiteUrl};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ImportSitesParams {
    /// Required unless the caller is an admin. Without it, an admin's sites are imported with no team.
    pub team_id: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct SiteImportRequest {
    pub version: u32,
    pub sites: Vec<ImportedSite>,
}

#[derive(Deserialize, ToSchema)]
pub struct ImportedSite {
    pub name: SiteName,
    pub url: SiteUrl,
    #[serde(default)]
    pub expected_status: ExpectedStatus,
    pub expected_text: Option<ExpectedText>,
    #[serde(default)]
    pub probe_interval_seconds: CheckInterval,
    #[serde(default)]
    pub tls_allow_untrusted: bool,
}
