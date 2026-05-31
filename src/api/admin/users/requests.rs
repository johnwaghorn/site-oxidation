use crate::models::user::UserRole;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
    /// Ignored for role `admin`.
    pub team_id: Option<i64>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub role: UserRole,
    pub active: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub temp_password: String,
}
