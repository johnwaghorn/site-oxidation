mod admin_queries;
mod teams_endpoints;
mod users_endpoints;

use axum::Router;
use axum::routing::{delete, get, patch, post};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{OpenApi, ToSchema};

use crate::api::errors::ApiError;
use crate::models::user::UserRole;
use crate::state::AppState;

use teams_endpoints::{
    add_team_member, create_team, delete_team, list_teams, remove_team_member, update_team,
};
use users_endpoints::{create_user, list_users, reset_password, update_user};

#[derive(OpenApi)]
#[openapi(
    paths(
        users_endpoints::list_users,
        users_endpoints::create_user,
        users_endpoints::update_user,
        users_endpoints::reset_password,
        teams_endpoints::list_teams,
        teams_endpoints::create_team,
        teams_endpoints::update_team,
        teams_endpoints::delete_team,
        teams_endpoints::add_team_member,
        teams_endpoints::remove_team_member,
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

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(list_users).post(create_user))
        .route("/admin/users/{id}", patch(update_user))
        .route("/admin/users/{id}/reset-password", post(reset_password))
        .route("/admin/teams", get(list_teams).post(create_team))
        .route("/admin/teams/{id}", patch(update_team).delete(delete_team))
        .route("/admin/teams/{id}/members", post(add_team_member))
        .route(
            "/admin/teams/{id}/members/{user_id}",
            delete(remove_team_member),
        )
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
    pub active: bool,
    pub must_change_password: bool,
    pub team_names: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub role: UserRole,
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub temp_password: String,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct TeamResponse {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
    pub site_count: i64,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTeamRequest {
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTeamRequest {
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
}
