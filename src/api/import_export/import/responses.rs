use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum ImportOutcome {
    Added,
    Skipped,
}

#[derive(Serialize, ToSchema)]
pub struct ImportedSiteResult {
    pub name: String,
    pub url: String,
    pub outcome: ImportOutcome,
}

#[derive(Serialize, ToSchema)]
pub struct SiteImportResponse {
    pub sites: Vec<ImportedSiteResult>,
}
