mod endpoints;
mod queries;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

use endpoints::export_sites;

pub fn import_export_routes() -> Router<AppState> {
    Router::new().route("/sites/export", get(export_sites))
}
