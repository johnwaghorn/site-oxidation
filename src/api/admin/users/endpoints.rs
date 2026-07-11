use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use password_auth::generate_hash;
use serde::Deserialize;
use sqlx::SqlitePool;
use tokio::task;

use super::params::ListUsersParams;
use super::queries;
use super::requests::{CreateUserRequest, ResetPasswordRequest, UpdateUserRequest};
use super::responses::{CreateUserResponse, UserResponse};
use super::rules;
use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::{
    ApiError, ApiErrorResponse, foreign_key_err, internal_err, unique_conflict_err,
};
use crate::api::extractors::{DeferredJsonPayload, JsonPayload, RequireAdmin};
use crate::api::pagination::{PaginatedResponse, PaginationParams, deserialize_u32_params};
use crate::api::search::{SearchParams, normalise_search};
use crate::models::user::UserRole;
use crate::security::password::{validate_password_bounds, validate_password_not_username};
use crate::state::AdminLimiter;

#[derive(Deserialize)]
pub(super) struct ListUsersQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    per_page: Option<u32>,
    search: Option<String>,
    team_id: Option<i64>,
    exclude_team_id: Option<i64>,
    active: Option<bool>,
}

#[utoipa::path(
    get,
    path = "/users",
    params(PaginationParams, SearchParams, ListUsersParams),
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
    let pagination = PaginationParams::new(params.page, params.per_page);
    let search = normalise_search(params.search.as_deref());
    let users = sqlx::query_as::<_, UserResponse>(queries::LIST_USERS)
        .bind(search)
        .bind(params.team_id)
        .bind(params.exclude_team_id)
        .bind(params.active)
        .bind(pagination.per_page())
        .bind(pagination.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list users", e))?;
    let total: i64 = sqlx::query_scalar(queries::COUNT_USERS)
        .bind(search)
        .bind(params.team_id)
        .bind(params.exclude_team_id)
        .bind(params.active)
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
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = CreateUserResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 409, description = "Username already exists", body = ApiError),
        (status = 422, description = "Validation error (e.g. missing team for a non-admin user)", body = ApiError),
        (status = 429, description = "Too many admin actions", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn create_user(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    State(AdminLimiter(limiter)): State<AdminLimiter>,
    payload: DeferredJsonPayload<CreateUserRequest>,
) -> Result<(StatusCode, Json<CreateUserResponse>), ApiErrorResponse> {
    let limiter_key = format!("create_user:{}", admin.id);
    if limiter.is_blocked(&limiter_key) {
        tracing::warn!(admin_id = admin.id, admin_username = %admin.username, action = "create_user", "Rate limited admin action");
        return Err(ApiErrorResponse::too_many_requests(
            "Too many admin actions, try again later",
        ));
    }
    limiter.record_failure(&limiter_key);
    let payload = payload.validated()?;
    validate_password_bounds(&payload.password)?;
    validate_password_not_username(&payload.password, payload.username.as_str())?;
    let team_id = rules::resolve_team_for_role(&pool, payload.role, payload.team_id).await?;

    let password = payload.password;
    let hash = task::spawn_blocking(move || generate_hash(&password))
        .await
        .map_err(|e| internal_err("Failed to hash password", e))?;
    let role_str = match payload.role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
    };
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| internal_err("Failed to start user creation", e))?;
    let id: i64 = sqlx::query_scalar(queries::INSERT_USER)
        .bind(payload.username.as_str())
        .bind(&hash)
        .bind(role_str)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| unique_conflict_err("Username already exists", "Failed to create user", e))?;
    if let Some(team_id) = team_id {
        sqlx::query(queries::ADD_TEAM_MEMBER)
            .bind(team_id)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| foreign_key_err("Team", "Failed to add user to team", e))?;
    }
    tx.commit()
        .await
        .map_err(|e| internal_err("Failed to commit user creation", e))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            id,
            username: payload.username.into_string(),
            role: payload.role,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "User not found", body = ApiError),
        (status = 409, description = "Cannot deactivate self or last admin, or demote an admin without a team", body = ApiError),
        (status = 422, description = "User payload validation error", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn update_user(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    JsonPayload(payload): JsonPayload<UpdateUserRequest>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    if let Some(message) = rules::self_update_error(&admin, id, &payload) {
        return Err(ApiErrorResponse::conflict(message));
    }
    rules::ensure_not_last_active_admin(&pool, id, &payload).await?;
    let role_str = match payload.role {
        UserRole::Admin => "admin",
        UserRole::User => "user",
    };
    let updated: Option<i64> = sqlx::query_scalar(queries::UPDATE_USER)
        .bind(role_str)
        .bind(payload.active)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to update user", e))?;

    rules::ensure_user_mutation_applied(
        &pool,
        id,
        updated,
        "Cannot demote an admin without assigning at least one team",
    )
    .await?;
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "User not found", body = ApiError),
        (status = 409, description = "Cannot delete self or last active admin", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/users",
    security(("session_cookie" = [])),
)]
pub async fn delete_user(
    RequireAdmin(admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiErrorResponse> {
    if let Some(message) = rules::self_delete_error(&admin, id) {
        return Err(ApiErrorResponse::conflict(message));
    }
    let deleted: Option<i64> = sqlx::query_scalar(queries::DELETE_USER)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to delete user", e))?;
    rules::ensure_user_mutation_applied(&pool, id, deleted, "Cannot delete the last active admin")
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/users/{id}/reset-password",
    params(("id" = i64, Path, description = "User ID")),
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "User not found", body = ApiError),
        (status = 422, description = "Password validation failed", body = ApiError),
        (status = 429, description = "Too many admin actions", body = ApiError),
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
    payload: DeferredJsonPayload<ResetPasswordRequest>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    let limiter_key = format!("reset_password:{}", admin.id);
    if limiter.is_blocked(&limiter_key) {
        tracing::warn!(admin_id = admin.id, admin_username = %admin.username, action = "reset_password", "Rate limited admin action");
        return Err(ApiErrorResponse::too_many_requests(
            "Too many admin actions, try again later",
        ));
    }
    limiter.record_failure(&limiter_key);
    let payload = payload.validated()?;
    validate_password_bounds(&payload.temp_password)?;
    let password = payload.temp_password;
    let hash = task::spawn_blocking(move || generate_hash(&password))
        .await
        .map_err(|e| internal_err("Failed to hash password", e))?;
    let updated: Option<i64> = sqlx::query_scalar(queries::RESET_PASSWORD)
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
