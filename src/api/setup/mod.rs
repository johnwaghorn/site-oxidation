mod endpoints;
mod queries;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use endpoints::{bootstrap, status};

pub fn setup_routes() -> Router<AppState> {
    Router::new()
        .route("/setup/status", get(status))
        .route("/setup/bootstrap", post(bootstrap))
}
