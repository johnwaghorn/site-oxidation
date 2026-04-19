mod endpoints_teams;
mod endpoints_users;
pub mod params;
mod queries_teams;
mod queries_users;
pub mod requests;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::{delete, get, patch, post};

use crate::state::AppState;

use endpoints_teams::{
    add_team_member, create_team, delete_team, list_teams, remove_team_member, update_team,
};
use endpoints_users::{create_user, list_users, reset_password, update_user};
use params::ListUsersParams;
use requests::{
    AddMemberRequest, CreateTeamRequest, CreateUserRequest, ResetPasswordRequest,
    UpdateTeamRequest, UpdateUserRequest,
};
use responses::{CreateUserResponse, SuccessResponse, TeamResponse, UserResponse};

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
