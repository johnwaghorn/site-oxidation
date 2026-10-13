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
