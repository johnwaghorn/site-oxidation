use super::requests::UpdateCanarySettingsRequest;
use super::responses::CanarySettingsResponse;
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::{JsonPayload, RequireAdmin};
use crate::canary;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;

async fn current_settings_response(
    state: &AppState,
) -> Result<Json<CanarySettingsResponse>, ApiErrorResponse> {
    let settings = canary::load_settings(&state.pool)
        .await
        .map_err(|error| internal_err("Failed to load canary settings", error))?;
    Ok(Json(settings.into()))
}

#[utoipa::path(
    get,
    path = "/canary",
    responses(
        (status = 200, description = "Global canary settings and health", body = CanarySettingsResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/canary",
    security(("session_cookie" = [])),
)]
pub async fn get_settings(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<CanarySettingsResponse>, ApiErrorResponse> {
    current_settings_response(&state).await
}

#[utoipa::path(
    put,
    path = "/canary",
    request_body = UpdateCanarySettingsRequest,
    responses(
        (status = 200, description = "Global canary settings updated", body = CanarySettingsResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 422, description = "Invalid canary URL or timeout", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/canary",
    security(("session_cookie" = [])),
)]
pub async fn update_settings(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
    JsonPayload(payload): JsonPayload<UpdateCanarySettingsRequest>,
) -> Result<Json<CanarySettingsResponse>, ApiErrorResponse> {
    let config = payload.into_canary_config()?;
    canary::update_settings(&state.pool, &config)
        .await
        .map_err(|error| internal_err("Failed to update canary settings", error))?;
    current_settings_response(&state).await
}

#[utoipa::path(
    post,
    path = "/canary/test",
    responses(
        (status = 200, description = "Canary test completed", body = CanarySettingsResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 409, description = "Settings changed while the test was running", body = ApiError),
        (status = 422, description = "Canary URL is not configured", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/canary",
    security(("session_cookie" = [])),
)]
pub async fn test_canary(
    RequireAdmin(_user): RequireAdmin,
    State(state): State<AppState>,
) -> Result<Json<CanarySettingsResponse>, ApiErrorResponse> {
    let settings = canary::load_settings(&state.pool)
        .await
        .map_err(|error| internal_err("Failed to load canary settings", error))?;
    if settings.config.url.is_none() {
        return Err(ApiErrorResponse::validation(
            "Configure a canary URL before testing",
        ));
    }
    let result = canary::check(&state.canary_client, &settings.config).await;
    let write = canary::record_result_if_current(&state.pool, settings.settings_revision, &result)
        .await
        .map_err(|error| internal_err("Failed to record canary result", error))?;
    match write {
        canary::CanaryResultWrite::Recorded => current_settings_response(&state).await,
        canary::CanaryResultWrite::DiscardedStaleRevision => Err(ApiErrorResponse::conflict(
            "Canary settings changed while the test was running; try again",
        )),
    }
}
