use axum::Json;
use axum::extract::{Path, State};

use super::queries;
use super::requests::UpdateTeamNotificationsRequest;
use super::responses::TeamNotificationsResponse;
use super::rules;
use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAppAccess;
use crate::api::teams::access::{ensure_team_access, ensure_team_exists};
use crate::models::smtp::SmtpSettings;
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
        (status = 422, description = "Invalid notification settings, e.g. a malformed webhook URL or an incomplete SMTP configuration", body = ApiError),
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
    let mut db_transaction = state
        .pool
        .begin()
        .await
        .map_err(|e| internal_err("Failed to update team notifications", e))?;
    let response = sqlx::query_as::<_, TeamNotificationsResponse>(queries::UPSERT_NOTIFICATIONS)
        .bind(id)
        .bind(update.slack_webhook_url.value.as_deref())
        .bind(update.microsoft_teams_webhook_url.value.as_deref())
        .bind(update.telegram_bot_token.value.as_deref())
        .bind(update.telegram_chat_id.value.as_deref())
        .bind(update.smtp_host.value.as_deref())
        .bind(update.smtp_port.value)
        .bind(update.smtp_tls_mode.value)
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
        .bind(update.smtp_tls_mode.provided)
        .bind(update.smtp_auth.provided)
        .bind(update.smtp_username.provided)
        .bind(update.smtp_password.provided)
        .bind(update.smtp_from_email.provided)
        .bind(update.smtp_to_email.provided)
        .bind(update.notify_site_down.provided)
        .bind(update.notify_site_recovered.provided)
        .bind(update.notify_cert_expiring.provided)
        .fetch_optional(&mut *db_transaction)
        .await
        .map_err(|e| internal_err("Failed to update team notifications", e))?
        .ok_or_else(|| ApiErrorResponse::not_found("Team"))?;
    if update.touches_smtp()
        && let Some(message) = rules::merged_smtp_config_error(&response)
    {
        return Err(ApiErrorResponse::validation(message));
    }
    db_transaction
        .commit()
        .await
        .map_err(|e| internal_err("Failed to update team notifications", e))?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/teams/{id}/notifications/test/email",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Test email sent via the team's SMTP settings", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Team membership required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 422, description = "SMTP is not configured for this team", body = ApiError),
        (status = 502, description = "The SMTP server rejected or did not accept the test email", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "teams",
    security(("session_cookie" = [])),
)]
pub async fn test_email_notification(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    ensure_team_exists(&state, id).await?;
    ensure_team_access(&state.pool, id, &user).await?;
    let smtp = sqlx::query_as::<_, SmtpSettings>(queries::SELECT_SMTP_SETTINGS)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to load SMTP settings", e))?;
    let Some(smtp) = smtp.filter(SmtpSettings::has_delivery_addresses) else {
        return Err(ApiErrorResponse::validation(
            "SMTP is not configured for this team",
        ));
    };
    state
        .notifier
        .test_email(&smtp, &user.username)
        .await
        .map_err(|error| ApiErrorResponse::bad_gateway(&format!("Test email failed: {error}")))?;
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    post,
    path = "/teams/{id}/notifications/test/slack",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Test message sent to the team's Slack webhook", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Team membership required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 422, description = "No Slack webhook is configured for this team", body = ApiError),
        (status = 502, description = "The Slack webhook rejected or did not accept the test message", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "teams",
    security(("session_cookie" = [])),
)]
pub async fn test_slack_notification(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    ensure_team_exists(&state, id).await?;
    ensure_team_access(&state.pool, id, &user).await?;
    let webhook_url: Option<Option<String>> = sqlx::query_scalar(queries::SELECT_SLACK_WEBHOOK_URL)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to load the Slack webhook", e))?;
    let Some(webhook_url) = webhook_url.flatten() else {
        return Err(ApiErrorResponse::validation(
            "No Slack webhook is configured for this team",
        ));
    };
    state
        .notifier
        .test_slack(&webhook_url, &user.username)
        .await
        .map_err(|error| {
            ApiErrorResponse::bad_gateway(&format!("Slack test message failed: {error}"))
        })?;
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    post,
    path = "/teams/{id}/notifications/test/teams",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Test message sent to the team's Microsoft Teams webhook", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Team membership required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 422, description = "No Microsoft Teams webhook is configured for this team", body = ApiError),
        (status = 502, description = "The Microsoft Teams webhook rejected or did not accept the test message", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "teams",
    security(("session_cookie" = [])),
)]
pub async fn test_teams_notification(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    ensure_team_exists(&state, id).await?;
    ensure_team_access(&state.pool, id, &user).await?;
    let webhook_url: Option<Option<String>> = sqlx::query_scalar(queries::SELECT_TEAMS_WEBHOOK_URL)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to load the Microsoft Teams webhook", e))?;
    let Some(webhook_url) = webhook_url.flatten() else {
        return Err(ApiErrorResponse::validation(
            "No Microsoft Teams webhook is configured for this team",
        ));
    };
    state
        .notifier
        .test_teams(&webhook_url, &user.username)
        .await
        .map_err(|error| {
            ApiErrorResponse::bad_gateway(&format!("Microsoft Teams test message failed: {error}"))
        })?;
    Ok(Json(SuccessResponse { success: true }))
}
