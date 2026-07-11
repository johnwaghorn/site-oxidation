mod endpoints;
mod fields;
pub mod params;
mod queries;
pub mod requests;
pub mod responses;
mod rules;
pub mod schema;

use axum::Router;
use axum::routing::{get, patch, post};

use crate::state::AppState;

use endpoints::{create_user, delete_user, list_users, reset_password, update_user};

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users).post(create_user))
        .route("/admin/users/{id}", patch(update_user).delete(delete_user))
        .route("/admin/users/{id}/reset-password", post(reset_password))
}
