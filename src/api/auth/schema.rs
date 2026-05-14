#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::requests::ChangePasswordRequest;
use super::responses::{ChangePasswordSuccess, LoginSuccess, MeSuccess};

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::login,
        endpoints::logout,
        endpoints::me,
        endpoints::change_password,
    ),
    components(schemas(
        crate::auth_backend::Credentials,
        LoginSuccess,
        MeSuccess,
        ChangePasswordRequest,
        ChangePasswordSuccess,
        ApiError,
    )),
    tags(
        (name = "auth", description = "Authentication"),
    ),
)]
pub struct AuthApiDoc;
