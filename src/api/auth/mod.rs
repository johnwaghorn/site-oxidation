mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;

use crate::state::AppState;
use axum::Router;
use axum::routing::{get, patch, post};

use endpoints::{change_password, login, logout, me, update_theme_preference};

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/auth/theme", patch(update_theme_preference))
        .route("/auth/change-password", post(change_password))
}
