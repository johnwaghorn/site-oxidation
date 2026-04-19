use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListUsersParams {
    /// Case-insensitive substring match against username.
    pub search: Option<String>,
    /// Restrict results to members of this team.
    pub team_id: Option<i64>,
}
