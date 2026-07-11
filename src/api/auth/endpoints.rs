use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use password_auth::{generate_hash, verify_password};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;

use super::requests::{ChangePasswordRequest, UpdateThemePreferenceRequest};
use super::responses::{
    ChangePasswordSuccess, LoginSuccess, MeSuccess, UpdateThemePreferenceSuccess, UserTeam,
};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::{JsonPayload, RequireAuth};
use crate::auth_backend::{AuthSession, Credentials};
use crate::models::user::UserRole;
use crate::security::password::{
    validate_password_bounds, validate_password_changed, validate_password_not_username,
};
use crate::security::rate_limit::LoginRateLimiter;

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = crate::auth_backend::Credentials,
    responses(
        (status = 200, description = "Login successful", body = LoginSuccess),
        (status = 401, description = "Invalid credentials", body = ApiError),
        (status = 422, description = "Credentials payload validation error", body = ApiError),
        (status = 429, description = "Too many login attempts", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "auth",
)]
pub async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(limiter): State<Arc<LoginRateLimiter>>,
    mut auth_session: AuthSession,
    JsonPayload(creds): JsonPayload<Credentials>,
) -> Result<Json<LoginSuccess>, ApiErrorResponse> {
    let attempted_username = creds.username.clone();
    let key = format!("{}:{}", addr.ip(), creds.username.to_lowercase());
    if limiter.is_blocked(&key) {
        return Err(ApiErrorResponse::too_many_requests(&format!(
            "Too many login attempts. Please wait {} seconds before trying again.",
            limiter.window_secs()
        )));
    }
    let user = auth_session
        .authenticate(creds)
        .await
        .map_err(|e| internal_err("Authentication failed", e))?;
    let user = if let Some(user) = user {
        limiter.clear(&key);
        user
    } else {
        tracing::warn!(
            "Failed login attempt for '{}' from {}",
            attempted_username,
            addr.ip()
        );
        let now_blocked = limiter.record_failure(&key);
        if now_blocked {
            tracing::warn!(
                "Login rate limit reached for {} (blocked for {}s)",
                addr.ip(),
                limiter.window_secs()
            );
        }
        return Err(ApiErrorResponse::unauthorized());
    };
    auth_session
        .login(&user)
        .await
        .map_err(|e| internal_err("Session login failed", e))?;
    tracing::info!("User '{}' logged in from {}", user.username, addr.ip());
    Ok(Json(LoginSuccess {
        username: user.username,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "auth",
    security(("session_cookie" = [])),
)]
pub async fn logout(mut auth_session: AuthSession) -> Result<StatusCode, ApiErrorResponse> {
    auth_session
        .logout()
        .await
        .map_err(|e| internal_err("Session logout failed", e))?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user info", body = MeSuccess),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "auth",
    security(("session_cookie" = [])),
)]
pub async fn me(
    auth_session: AuthSession,
    State(pool): State<SqlitePool>,
) -> Result<Json<MeSuccess>, ApiErrorResponse> {
    let user = auth_session
        .user
        .ok_or_else(ApiErrorResponse::unauthorized)?;
    let teams = match user.role {
        UserRole::Admin => {
            sqlx::query_as::<_, UserTeam>(super::queries::SELECT_ALL_TEAMS)
                .fetch_all(&pool)
                .await
        }
        UserRole::User => {
            sqlx::query_as::<_, UserTeam>(super::queries::SELECT_USER_TEAMS)
                .bind(user.id)
                .fetch_all(&pool)
                .await
        }
    }
    .map_err(|e| internal_err("Failed to fetch user teams", e))?;
    Ok(Json(MeSuccess {
        id: user.id,
        username: user.username,
        role: user.role,
        must_change_password: user.must_change_password,
        theme_preference: user.theme_preference,
        teams,
    }))
}

#[utoipa::path(
    patch,
    path = "/auth/theme",
    request_body = UpdateThemePreferenceRequest,
    responses(
        (status = 200, description = "Theme preference updated", body = UpdateThemePreferenceSuccess),
        (status = 401, description = "Not authenticated", body = ApiError),
        (status = 422, description = "Theme preference validation failed", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "auth",
    security(("session_cookie" = [])),
)]
pub async fn update_theme_preference(
    RequireAuth(user): RequireAuth,
    State(pool): State<SqlitePool>,
    JsonPayload(payload): JsonPayload<UpdateThemePreferenceRequest>,
) -> Result<Json<UpdateThemePreferenceSuccess>, ApiErrorResponse> {
    sqlx::query(super::queries::UPDATE_THEME_PREFERENCE)
        .bind(payload.theme_preference)
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| {
            internal_err(
                &format!("Failed to update theme preference for user {}", user.id),
                e,
            )
        })?;

    Ok(Json(UpdateThemePreferenceSuccess {
        theme_preference: payload.theme_preference,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed", body = ChangePasswordSuccess),
        (status = 401, description = "Invalid current password", body = ApiError),
        (status = 422, description = "Password validation failed", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "auth",
    security(("session_cookie" = [])),
)]
pub async fn change_password(
    RequireAuth(user): RequireAuth,
    State(pool): State<SqlitePool>,
    mut auth_session: AuthSession,
    JsonPayload(payload): JsonPayload<ChangePasswordRequest>,
) -> Result<Json<ChangePasswordSuccess>, ApiErrorResponse> {
    validate_password_bounds(&payload.new_password)?;
    validate_password_not_username(&payload.new_password, &user.username)?;
    validate_password_changed(&payload.new_password, &payload.current_password)?;
    let current_password = payload.current_password;
    let stored_hash = user.password.clone();
    let is_valid = tokio::task::spawn_blocking(move || {
        verify_password(&current_password, &stored_hash).is_ok()
    })
    .await
    .map_err(|e| internal_err("Failed to verify current password", e))?;
    if !is_valid {
        return Err(ApiErrorResponse::unauthorized());
    }
    let new_password = payload.new_password;
    let new_hash = tokio::task::spawn_blocking(move || generate_hash(&new_password))
        .await
        .map_err(|e| internal_err("Failed to hash new password", e))?;
    sqlx::query(super::queries::UPDATE_PASSWORD)
        .bind(&new_hash)
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|e| {
            internal_err(
                &format!("Failed to update password for user {}", user.id),
                e,
            )
        })?;
    let updated_user =
        sqlx::query_as::<_, crate::models::user::User>(super::queries::SELECT_USER_BY_ID)
            .bind(user.id)
            .fetch_one(&pool)
            .await
            .map_err(|e| internal_err("Failed to re-fetch user after password change", e))?;
    auth_session
        .login(&updated_user)
        .await
        .map_err(|e| internal_err("Failed to refresh session after password change", e))?;
    Ok(Json(ChangePasswordSuccess { success: true }))
}
