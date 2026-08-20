mod endpoints;
pub mod requests;
pub mod responses;
pub mod schema;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};
use endpoints::{get_settings, test_canary, update_settings};

pub fn canary_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/canary", get(get_settings).put(update_settings))
        .route("/admin/canary/test", post(test_canary))
}
