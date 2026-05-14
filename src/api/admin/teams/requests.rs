use serde::Deserialize;
use utoipa::ToSchema;

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
