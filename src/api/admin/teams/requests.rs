use super::fields::TeamName;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateTeamRequest {
    pub name: TeamName,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTeamRequest {
    pub name: TeamName,
}

#[derive(Deserialize, ToSchema)]
pub struct AddMemberRequest {
    pub user_id: i64,
}
