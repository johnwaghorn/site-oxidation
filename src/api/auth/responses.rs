use crate::models::user::{ThemePreference, UserRole};
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
    pub theme_preference: ThemePreference,
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

#[derive(Serialize, ToSchema)]
pub struct UpdateThemePreferenceSuccess {
    pub theme_preference: ThemePreference,
}
