use axum::Router;
use axum::routing::post;

use crate::state::AppState;

use endpoints::import_sites;

mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;

pub fn import_routes() -> Router<AppState> {
    Router::new().route("/sites/import", post(import_sites))
}
