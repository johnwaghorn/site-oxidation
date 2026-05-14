mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, post};

use endpoints::{change_password, login, logout, me};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/auth/change-password", post(change_password))
}
