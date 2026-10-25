use crate::api::pagination::deserialize_u32_params;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListUsersParams {
    /// Restrict results to members of this team.
    pub team_id: Option<i64>,
    /// Exclude members of this team from results.
    pub exclude_team_id: Option<i64>,
    /// Restrict results by active status.
    pub active: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListUsersQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub team_id: Option<i64>,
    pub exclude_team_id: Option<i64>,
    pub active: Option<bool>,
}
