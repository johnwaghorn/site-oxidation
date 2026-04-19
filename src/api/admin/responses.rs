use crate::models::user::UserRole;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Serialize, FromRow, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
    pub active: bool,
    pub must_change_password: bool,
    pub team_names: String,
}

#[derive(Serialize, ToSchema)]
pub struct CreateUserResponse {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct TeamResponse {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
    pub site_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct SuccessResponse {
    pub success: bool,
}
