mod access;
mod endpoints;
mod fields;
mod queries;
pub mod requests;
pub mod responses;
mod rules;
pub mod schema;

use crate::state::AppState;
use axum::{Router, routing::get};

use endpoints::{create_site, delete_site, get_site, get_site_outages, list_sites, update_site};
use requests::SitePayload;
use responses::{OutageResponse, SiteResponse};

pub fn site_routes() -> Router<AppState> {
    Router::new()
        .route("/sites", get(list_sites).post(create_site))
        .route(
            "/sites/{id}",
            get(get_site).put(update_site).delete(delete_site),
        )
        .route("/sites/{id}/outages", get(get_site_outages))
}
