#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::ApiError;

use super::endpoints;
use super::requests::{CreateUserRequest, ResetPasswordRequest, UpdateUserRequest};
use super::responses::{CreateUserResponse, UserResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::list_users,
        endpoints::create_user,
        endpoints::update_user,
        endpoints::reset_password,
    ),
    components(schemas(
        UserResponse,
        CreateUserRequest,
        CreateUserResponse,
        UpdateUserRequest,
        ResetPasswordRequest,
        SuccessResponse,
        ApiError,
    )),
    tags(
        (name = "admin/users", description = "User management (admin only)"),
    ),
)]
pub struct UsersApiDoc;
