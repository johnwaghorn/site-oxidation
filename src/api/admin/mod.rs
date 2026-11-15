use axum::Router;

use crate::state::AppState;

pub mod canary;
pub mod notification_outbox;
pub mod responses;
pub mod schema;
pub mod teams;
pub mod users;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .merge(users::user_routes())
        .merge(teams::team_routes())
        .merge(canary::canary_routes())
        .merge(notification_outbox::notification_outbox_routes())
}
