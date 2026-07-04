use axum::Json;
use axum::extract::{Path, State};

use super::queries;
use super::requests::UpdateTeamNotificationsRequest;
use super::responses::TeamNotificationsResponse;
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAppAccess;
use crate::api::teams::access::{ensure_team_access, ensure_team_exists};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/teams/{id}/notifications",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Team notification settings", body = TeamNotificationsResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Team membership required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "teams",
    security(("session_cookie" = [])),
)]
pub async fn get_notifications(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TeamNotificationsResponse>, ApiErrorResponse> {
    ensure_team_exists(&state, id).await?;
    ensure_team_access(&state.pool, id, &user).await?;
    let response = sqlx::query_as::<_, TeamNotificationsResponse>(queries::SELECT_NOTIFICATIONS)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to get team notifications", e))?
        .ok_or_else(|| ApiErrorResponse::not_found("Team"))?;
    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/teams/{id}/notifications",
    params(("id" = i64, Path, description = "Team ID")),
    request_body = UpdateTeamNotificationsRequest,
    responses(
        (status = 200, description = "Team notification settings updated", body = TeamNotificationsResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Team membership required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 422, description = "Invalid Slack webhook URL", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "teams",
    security(("session_cookie" = [])),
)]
pub async fn update_notifications(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTeamNotificationsRequest>,
) -> Result<Json<TeamNotificationsResponse>, ApiErrorResponse> {
    ensure_team_exists(&state, id).await?;
    ensure_team_access(&state.pool, id, &user).await?;
    let update = payload.prepare().map_err(ApiErrorResponse::validation)?;
    let response = sqlx::query_as::<_, TeamNotificationsResponse>(queries::UPSERT_NOTIFICATIONS)
        .bind(id)
        .bind(update.slack_webhook_url.value.as_deref())
        .bind(update.microsoft_teams_webhook_url.value.as_deref())
        .bind(update.telegram_bot_token.value.as_deref())
        .bind(update.telegram_chat_id.value.as_deref())
        .bind(update.smtp_host.value.as_deref())
        .bind(update.smtp_port.value)
        .bind(update.smtp_security.value.as_deref())
        .bind(update.smtp_auth.value)
        .bind(update.smtp_username.value.as_deref())
        .bind(update.smtp_password.value.as_deref())
        .bind(update.smtp_from_email.value.as_deref())
        .bind(update.smtp_to_email.value.as_deref())
        .bind(update.notify_site_down.value)
        .bind(update.notify_site_recovered.value)
        .bind(update.notify_cert_expiring.value)
        .bind(update.slack_webhook_url.provided)
        .bind(update.microsoft_teams_webhook_url.provided)
        .bind(update.telegram_bot_token.provided)
        .bind(update.telegram_chat_id.provided)
        .bind(update.smtp_host.provided)
        .bind(update.smtp_port.provided)
        .bind(update.smtp_security.provided)
        .bind(update.smtp_auth.provided)
        .bind(update.smtp_username.provided)
        .bind(update.smtp_password.provided)
        .bind(update.smtp_from_email.provided)
        .bind(update.smtp_to_email.provided)
        .bind(update.notify_site_down.provided)
        .bind(update.notify_site_recovered.provided)
        .bind(update.notify_cert_expiring.provided)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to update team notifications", e))?
        .ok_or_else(|| ApiErrorResponse::not_found("Team"))?;
    Ok(Json(response))
}
