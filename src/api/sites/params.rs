use crate::api::pagination::deserialize_u32_params;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

/// Status buckets. `down` also matches blocked sites, since both mean the site needs attention.
#[derive(Debug, Clone, Copy, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SiteStatusFilter {
    Up,
    Down,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListSitesParams {
    /// Restrict results by status. `down` includes blocked sites.
    pub status: Option<SiteStatusFilter>,
}

#[derive(Deserialize)]
pub struct ListSitesQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<SiteStatusFilter>,
}
