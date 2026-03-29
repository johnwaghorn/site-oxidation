use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use password_auth::generate_hash;
use sqlx::SqlitePool;
use tokio::task;

use super::{
    CreateUserRequest, CreateUserResponse, ResetPasswordRequest, SuccessResponse,
    UpdateUserRequest, UserResponse, admin_queries,
};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAdmin;
use crate::models::user::UserRole;
use crate::security::password::{validate_password_bounds, validate_password_not_username};
use crate::state::AdminLimiter;

#[utoipa::path(
    get,
    path = "/admin/users",
    responses(
        (status = 200, description = "List all users", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn list_users(
    RequireAdmin(_user): RequireAdmin,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<UserResponse>>, ApiErrorResponse> {
    let users = sqlx::query_as::<_, UserResponse>(admin_queries::LIST_USERS)
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list users", e))?;
    Ok(Json(users))
}

#[utoipa::path(
    post,
    path = "/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = CreateUserResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 409, description = "Username already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn create_user(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    State(AdminLimiter(limiter)): State<AdminLimiter>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<CreateUserResponse>), ApiErrorResponse> {
    let limiter_key = format!("create_user:{}", admin.id);
    if limiter.is_blocked(&limiter_key) {
        return Err(ApiErrorResponse::too_many_requests(
            "Too many admin actions, try again later",
        ));
    }
    limiter.record_failure(&limiter_key);
    let username = payload.username.trim().to_owned();
    if username.is_empty() || username.len() > 100 {
        return Err(ApiErrorResponse::validation(
            "Username must be between 1 and 100 characters",
        ));
    }
    validate_password_bounds(&payload.password)?;
    validate_password_not_username(&payload.password, &username)?;
    let password = payload.password;
    let hash = task::spawn_blocking(move || generate_hash(&password))
        .await
        .map_err(|e| internal_err("Failed to hash password", e))?;
    let role_str = match payload.role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
    };
    let id: i64 = sqlx::query_scalar(admin_queries::INSERT_USER)
        .bind(&username)
        .bind(&hash)
        .bind(role_str)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                ApiErrorResponse::conflict("Username already exists")
            } else {
                internal_err("Failed to create user", e)
            }
        })?;
    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            id,
            username,
            role: payload.role,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/admin/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "User not found", body = ApiError),
        (status = 409, description = "Cannot deactivate self or last admin", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn update_user(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    if id == admin.id && !payload.active {
        return Err(ApiErrorResponse::conflict(
            "Cannot deactivate your own account",
        ));
    }
    if id == admin.id && payload.role != UserRole::Admin {
        return Err(ApiErrorResponse::conflict("Cannot demote your own account"));
    }
    if payload.role != UserRole::Admin || !payload.active {
        let active_admins: i64 = sqlx::query_scalar(admin_queries::COUNT_ACTIVE_ADMINS)
            .fetch_one(&pool)
            .await
            .map_err(|e| internal_err("Failed to count active admins", e))?;
        let is_target_active_admin: bool = sqlx::query_scalar(admin_queries::IS_ACTIVE_ADMIN)
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| internal_err("Failed to check user status", e))?
            .unwrap_or(false);
        if is_target_active_admin && active_admins <= 1 {
            return Err(ApiErrorResponse::conflict(
                "Cannot demote or deactivate the last active admin",
            ));
        }
    }
    let role_str = match payload.role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
    };
    let updated: Option<i64> = sqlx::query_scalar(admin_queries::UPDATE_USER)
        .bind(role_str)
        .bind(payload.active)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to update user", e))?;

    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("User"));
    }
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    post,
    path = "/admin/users/{id}/reset-password",
    params(("id" = i64, Path, description = "User ID")),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "User not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn reset_password(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    State(AdminLimiter(limiter)): State<AdminLimiter>,
    Path(id): Path<i64>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    let limiter_key = format!("reset_password:{}", admin.id);
    if limiter.is_blocked(&limiter_key) {
        return Err(ApiErrorResponse::too_many_requests(
            "Too many admin actions, try again later",
        ));
    }
    limiter.record_failure(&limiter_key);
    validate_password_bounds(&payload.temp_password)?;
    let password = payload.temp_password;
    let hash = task::spawn_blocking(move || generate_hash(&password))
        .await
        .map_err(|e| internal_err("Failed to hash password", e))?;
    let updated: Option<i64> = sqlx::query_scalar(admin_queries::RESET_PASSWORD)
        .bind(&hash)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to reset password", e))?;
    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("User"));
    }
    Ok(Json(SuccessResponse { success: true }))
}
