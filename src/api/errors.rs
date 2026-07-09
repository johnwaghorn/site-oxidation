use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt::Display;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    pub error: String,
    pub message: String,
}

pub struct ApiErrorResponse {
    status: StatusCode,
    body: ApiError,
}

pub fn internal_err<E>(msg: &str, e: E) -> ApiErrorResponse
where
    E: Display,
{
    tracing::error!("{msg}: {e}");
    ApiErrorResponse::internal(msg)
}

pub fn unique_conflict_err(conflict_msg: &str, context: &str, e: sqlx::Error) -> ApiErrorResponse {
    if e.as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
    {
        return ApiErrorResponse::conflict(conflict_msg);
    }
    internal_err(context, e)
}

pub fn foreign_key_err(resource: &str, context: &str, e: sqlx::Error) -> ApiErrorResponse {
    if e.as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_foreign_key_violation)
    {
        return ApiErrorResponse::not_found(resource);
    }
    internal_err(context, e)
}

impl ApiErrorResponse {
    pub fn not_found(resource: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            body: ApiError {
                error: "not_found".to_owned(),
                message: format!("{resource} not found"),
            },
        }
    }

    pub fn internal(message: &str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiError {
                error: "internal_error".to_owned(),
                message: message.to_owned(),
            },
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: ApiError {
                error: "unauthorized".to_owned(),
                message: "Authentication required".to_owned(),
            },
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            body: ApiError {
                error: "forbidden".to_owned(),
                message: message.to_owned(),
            },
        }
    }

    pub fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            body: ApiError {
                error: "conflict".to_owned(),
                message: message.to_owned(),
            },
        }
    }

    pub fn too_many_requests(message: &str) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            body: ApiError {
                error: "too_many_requests".to_owned(),
                message: message.to_owned(),
            },
        }
    }

    pub fn validation(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: ApiError {
                error: "validation_error".to_owned(),
                message: message.to_owned(),
            },
        }
    }

    pub fn bad_gateway(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            body: ApiError {
                error: "bad_gateway".to_owned(),
                message: message.to_owned(),
            },
        }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}
