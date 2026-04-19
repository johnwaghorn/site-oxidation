use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints_teams;
use super::endpoints_users;
use super::requests::{
    AddMemberRequest, CreateTeamRequest, CreateUserRequest, ResetPasswordRequest,
    UpdateTeamRequest, UpdateUserRequest,
};
use super::responses::{CreateUserResponse, SuccessResponse, TeamResponse, UserResponse};

#[allow(clippy::needless_for_each)]
#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints_users::list_users,
        endpoints_users::create_user,
        endpoints_users::update_user,
        endpoints_users::reset_password,
        endpoints_teams::list_teams,
        endpoints_teams::create_team,
        endpoints_teams::update_team,
        endpoints_teams::delete_team,
        endpoints_teams::add_team_member,
        endpoints_teams::remove_team_member,
    ),
    components(schemas(
        UserResponse,
        CreateUserRequest,
        CreateUserResponse,
        UpdateUserRequest,
        ResetPasswordRequest,
        TeamResponse,
        CreateTeamRequest,
        UpdateTeamRequest,
        AddMemberRequest,
        SuccessResponse,
        ApiError,
    )),
    tags(
        (name = "admin/users", description = "User management (admin only)"),
        (name = "admin/teams", description = "Team management (admin only)"),
    ),
)]
pub struct AdminApiDoc;
