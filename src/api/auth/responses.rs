use crate::models::user::UserRole;
use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginSuccess {
    pub username: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeSuccess {
    pub id: i64,
    pub username: String,
    pub role: UserRole,
    pub must_change_password: bool,
    pub teams: Vec<UserTeam>,
}

#[derive(Debug, Serialize, FromRow, ToSchema)]
pub struct UserTeam {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct ChangePasswordSuccess {
    pub success: bool,
}
