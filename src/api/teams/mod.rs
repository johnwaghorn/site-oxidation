pub(crate) mod access;
mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;
mod validators;

use axum::Router;
use axum::routing::get;

use crate::state::AppState;

use endpoints::{get_notifications, update_notifications};

pub fn team_routes() -> Router<AppState> {
    Router::new().route(
        "/teams/{id}/notifications",
        get(get_notifications).patch(update_notifications),
    )
}
