pub mod responses;
pub mod schema;
pub mod teams;
pub mod users;

use axum::Router;

use crate::state::AppState;

pub fn admin_routes() -> Router<AppState> {
    Router::new()
        .merge(users::user_routes())
        .merge(teams::team_routes())
}
