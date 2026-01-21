mod sites_endpoints;
mod sites_validators;

use super::errors::ApiError;
use super::pagination::PaginatedResponse;
use crate::models::SiteStatus;
use crate::state::AppState;
use axum::{Router, routing::get};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sites_endpoints::{
    create_site, delete_site, get_site, get_site_outages, list_sites, update_site,
};
use sites_validators::{ExpectedStatus, ExpectedText, SiteName, SiteUrl};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        sites_endpoints::list_sites,
        sites_endpoints::get_site,
        sites_endpoints::create_site,
        sites_endpoints::update_site,
        sites_endpoints::delete_site,
        sites_endpoints::get_site_outages
    ),
    components(schemas(SiteResponse, PaginatedResponse<SiteResponse>, OutageResponse, PaginatedResponse<OutageResponse>, SitePayload, SiteStatus, ApiError)),
    tags(
          (name = "sites", description = "Site monitoring"),
    ),
)]
pub struct SitesApiDoc;

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

#[derive(Deserialize, ToSchema)]
pub struct SitePayload {
    pub name: SiteName,
    pub url: SiteUrl,
    #[serde(default)]
    pub expected_status: ExpectedStatus,
    pub expected_text: Option<ExpectedText>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sites", get(list_sites).post(create_site))
        .route(
            "/sites/{id}",
            get(get_site).put(update_site).delete(delete_site),
        )
        .route("/sites/{id}/outages", get(get_site_outages))
}
