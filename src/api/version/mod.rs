use axum::Router;
use axum::routing::get;

use crate::state::AppState;

use endpoints::version;

mod endpoints;
pub mod responses;
pub mod schema;

pub fn version_routes() -> Router<AppState> {
    Router::new().route("/version", get(version))
}
