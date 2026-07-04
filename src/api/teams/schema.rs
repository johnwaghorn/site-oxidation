#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::endpoints;
use super::requests::UpdateTeamNotificationsRequest;
use super::responses::TeamNotificationsResponse;
use crate::api::errors::ApiError;

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::get_notifications,
        endpoints::update_notifications,
    ),
    components(schemas(
        TeamNotificationsResponse,
        UpdateTeamNotificationsRequest,
        ApiError,
    )),
    tags(
        (name = "teams", description = "Team settings"),
    ),
)]
pub struct TeamsApiDoc;
