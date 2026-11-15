use crate::state::AppState;
use axum::Router;
use axum::routing::{delete, get};
use endpoints::{cancel_notification_outbox_entry, list_notification_outbox_entries};

mod endpoints;
pub mod responses;
pub mod schema;

pub fn notification_outbox_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/admin/notification-outbox",
            get(list_notification_outbox_entries),
        )
        .route(
            "/admin/notification-outbox/{id}",
            delete(cancel_notification_outbox_entry),
        )
}
