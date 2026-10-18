mod endpoints;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

use endpoints::health;

pub fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
