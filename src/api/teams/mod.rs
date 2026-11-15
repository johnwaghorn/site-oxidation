use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use endpoints::{
    get_notifications, list_team_notification_summaries, test_email_notification,
    test_slack_notification, test_teams_notification, update_notifications,
};

pub(crate) mod access;
mod endpoints;
mod fields;
pub mod params;
mod queries;
pub mod requests;
pub mod responses;
mod rules;
pub mod schema;

pub fn team_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/teams/notifications",
            get(list_team_notification_summaries),
        )
        .route(
            "/teams/{id}/notifications",
            get(get_notifications).patch(update_notifications),
        )
        .route(
            "/teams/{id}/notifications/test/email",
            post(test_email_notification),
        )
        .route(
            "/teams/{id}/notifications/test/slack",
            post(test_slack_notification),
        )
        .route(
            "/teams/{id}/notifications/test/teams",
            post(test_teams_notification),
        )
}
