#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::requests::{ChangePasswordRequest, UpdateThemePreferenceRequest};
use super::responses::{
    ChangePasswordSuccess, LoginSuccess, MeSuccess, UpdateThemePreferenceSuccess, UserTeam,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::login,
        endpoints::logout,
        endpoints::me,
        endpoints::update_theme_preference,
        endpoints::change_password,
    ),
    components(schemas(
        crate::auth_backend::Credentials,
        crate::models::user::ThemePreference,
        LoginSuccess,
        MeSuccess,
        UserTeam,
        ChangePasswordRequest,
        ChangePasswordSuccess,
        UpdateThemePreferenceRequest,
        UpdateThemePreferenceSuccess,
        ApiError,
    )),
    tags(
        (name = "auth", description = "Authentication"),
    ),
)]
pub struct AuthApiDoc;
