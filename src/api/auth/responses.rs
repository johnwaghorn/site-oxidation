use crate::models::user::UserRole;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginSuccess {
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeSuccess {
    pub username: String,
    pub role: UserRole,
    pub must_change_password: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ChangePasswordSuccess {
    pub success: bool,
}
