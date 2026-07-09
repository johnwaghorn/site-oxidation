#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::endpoints;
use super::requests::UpdateTeamNotificationsRequest;
use super::responses::TeamNotificationsResponse;
use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::ApiError;

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::get_notifications,
        endpoints::update_notifications,
        endpoints::test_email_notification,
        endpoints::test_slack_notification,
        endpoints::test_teams_notification,
    ),
    components(schemas(
        TeamNotificationsResponse,
        UpdateTeamNotificationsRequest,
        SuccessResponse,
        ApiError,
    )),
    tags(
        (name = "teams", description = "Team settings"),
    ),
)]
pub struct TeamsApiDoc;
