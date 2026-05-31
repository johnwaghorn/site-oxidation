#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::ApiError;

use super::endpoints;
use super::requests::{AddMemberRequest, CreateTeamRequest, UpdateTeamRequest};
use super::responses::{TeamOption, TeamResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::list_teams,
        endpoints::list_team_options,
        endpoints::create_team,
        endpoints::update_team,
        endpoints::delete_team,
        endpoints::add_team_member,
        endpoints::remove_team_member,
    ),
    components(schemas(
        TeamResponse,
        TeamOption,
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
