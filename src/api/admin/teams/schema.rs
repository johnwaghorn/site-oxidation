#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::ApiError;

use super::endpoints;
use super::requests::{AddMemberRequest, CreateTeamRequest, UpdateTeamRequest};
use super::responses::TeamResponse;

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::list_teams,
        endpoints::create_team,
        endpoints::update_team,
        endpoints::delete_team,
        endpoints::add_team_member,
        endpoints::remove_team_member,
    ),
    components(schemas(
        TeamResponse,
        CreateTeamRequest,
        UpdateTeamRequest,
        AddMemberRequest,
        SuccessResponse,
        ApiError,
    )),
    tags(
        (name = "admin/teams", description = "Team management (admin only)"),
    ),
)]
pub struct TeamsApiDoc;
