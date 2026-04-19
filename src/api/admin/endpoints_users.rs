use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use password_auth::generate_hash;
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::task;

use super::{
    CreateUserRequest, CreateUserResponse, ListUsersParams, ResetPasswordRequest, SuccessResponse,
    UpdateUserRequest, UserResponse, queries_users,
};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAdmin;
use crate::api::pagination::{PaginatedResponse, PaginationParams};
use crate::models::user::UserRole;
use crate::security::password::{validate_password_bounds, validate_password_not_username};
use crate::state::AdminLimiter;

#[derive(Deserialize)]
pub(super) struct ListUsersQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    team_id: Option<i64>,
}

impl ListUsersQuery {
    fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            per_page: self.per_page,
        }
    }

    fn search(&self) -> Option<String> {
        self.search
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
    }
}

#[utoipa::path(
    get,
    path = "/admin/users",
    params(PaginationParams, ListUsersParams),
    responses(
        (status = 200, description = "List all users", body = inline(PaginatedResponse<UserResponse>)),
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
    Query(params): Query<ListUsersQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, ApiErrorResponse> {
    let pagination = params.pagination();
    let search = params.search();
    let users = sqlx::query_as::<_, UserResponse>(queries_users::LIST_USERS)
        .bind(search.as_deref())
        .bind(params.team_id)
        .bind(pagination.per_page())
        .bind(pagination.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list users", e))?;
    let total: i64 = sqlx::query_scalar(queries_users::COUNT_USERS)
        .bind(search.as_deref())
        .bind(params.team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to count users", e))?;
    Ok(Json(PaginatedResponse {
        data: users,
        page: pagination.page(),
        per_page: pagination.per_page(),
        total,
    }))
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
        tracing::warn!(admin_id = admin.id, admin_username = %admin.username, action = "create_user", "Rate limited admin action");
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
    let id: i64 = sqlx::query_scalar(queries_users::INSERT_USER)
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
        let active_admins: i64 = sqlx::query_scalar(queries_users::COUNT_ACTIVE_ADMINS)
            .fetch_one(&pool)
            .await
            .map_err(|e| internal_err("Failed to count active admins", e))?;
        let is_target_active_admin: bool = sqlx::query_scalar(queries_users::IS_ACTIVE_ADMIN)
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
    let updated: Option<i64> = sqlx::query_scalar(queries_users::UPDATE_USER)
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
        tracing::warn!(admin_id = admin.id, admin_username = %admin.username, action = "reset_password", "Rate limited admin action");
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
    let updated: Option<i64> = sqlx::query_scalar(queries_users::RESET_PASSWORD)
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
