use crate::models::user::UserRole;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: UserRole,
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
