use crate::state::AppState;
use axum::{Router, routing::get};

use endpoints::{
    count_unassigned_sites, create_site, delete_site, get_site, get_site_outages, get_site_uptime,
    list_sites, update_site,
};
use requests::SitePayload;
use responses::{OutageResponse, SiteResponse};

pub(crate) mod access;
pub(crate) mod canary_warning;
mod endpoints;
pub(crate) mod fields;
pub mod params;
mod queries;
pub mod requests;
pub mod responses;
pub(crate) mod rules;
pub mod schema;
pub(crate) mod uptime;

pub fn site_routes() -> Router<AppState> {
    Router::new()
        .route("/sites", get(list_sites).post(create_site))
        .route("/sites/unassigned-count", get(count_unassigned_sites))
        .route(
            "/sites/{id}",
            get(get_site).put(update_site).delete(delete_site),
        )
        .route("/sites/{id}/outages", get(get_site_outages))
        .route("/sites/{id}/uptime", get(get_site_uptime))
}
