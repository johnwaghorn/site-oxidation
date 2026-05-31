mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::{delete, get, patch, post};

use crate::state::AppState;

use endpoints::{
    add_team_member, create_team, delete_team, list_team_options, list_teams, remove_team_member,
    update_team,
};

pub fn team_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/teams", get(list_teams).post(create_team))
        .route("/admin/teams/options", get(list_team_options))
        .route("/admin/teams/{id}", patch(update_team).delete(delete_team))
        .route("/admin/teams/{id}/members", post(add_team_member))
        .route(
            "/admin/teams/{id}/members/{user_id}",
            delete(remove_team_member),
        )
}
