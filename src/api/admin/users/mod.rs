use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use endpoints::{create_user, delete_user, get_user, list_users, reset_password, update_user};

mod endpoints;
mod fields;
pub mod params;
mod queries;
pub mod requests;
pub mod responses;
mod rules;
pub mod schema;

pub fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users).post(create_user))
        .route(
            "/admin/users/{id}",
            get(get_user).patch(update_user).delete(delete_user),
        )
        .route("/admin/users/{id}/reset-password", post(reset_password))
}
