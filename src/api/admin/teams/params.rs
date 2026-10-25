use crate::api::pagination::deserialize_u32_params;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListTeamsParams {
    /// Restrict results to teams this user belongs to.
    pub member_id: Option<i64>,
    /// Exclude teams this user already belongs to.
    pub exclude_member_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct ListTeamsQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub member_id: Option<i64>,
    pub exclude_member_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct ListTeamSitesQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
}
