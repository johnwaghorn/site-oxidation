mod endpoints;
mod queries;
pub mod requests;
pub mod responses;
pub mod schema;

use axum::Router;
use axum::routing::{delete, get, post};

use crate::state::AppState;

use endpoints::{
    add_team_member, create_team, delete_team, get_team, list_team_options, list_team_sites,
    list_teams, remove_team_member, unassign_team_site, update_team,
};

pub fn team_routes() -> Router<AppState> {
    Router::new()
        .route("/admin/teams", get(list_teams).post(create_team))
        .route("/admin/teams/options", get(list_team_options))
        .route(
            "/admin/teams/{id}",
            get(get_team).patch(update_team).delete(delete_team),
        )
        .route("/admin/teams/{id}/sites", get(list_team_sites))
        .route(
            "/admin/teams/{id}/sites/{site_id}",
            delete(unassign_team_site),
        )
        .route("/admin/teams/{id}/members", post(add_team_member))
        .route(
            "/admin/teams/{id}/members/{user_id}",
            delete(remove_team_member),
        )
}
