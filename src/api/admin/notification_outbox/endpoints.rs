use super::responses::NotificationOutboxEntryResponse;
use crate::api::action_log::{id_or_none, log_action_by_user};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAdmin;
use crate::api::pagination::{PaginatedResponse, PaginationParams};
use crate::notifications::outbox::{self, CancelOutcome};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use sqlx::SqlitePool;

#[utoipa::path(
    get,
    path = "/notification-outbox",
    params(PaginationParams),
    responses(
        (status = 200, description = "Notifications waiting to be sent, oldest first", body = inline(PaginatedResponse<NotificationOutboxEntryResponse>)),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/notification-outbox",
    security(("session_cookie" = [])),
)]
pub async fn list_notification_outbox_entries(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<NotificationOutboxEntryResponse>>, ApiErrorResponse> {
    let entries = outbox::list_entries_oldest_first(&pool, params.per_page(), params.offset())
        .await
        .map_err(|e| internal_err("Failed to load the notification outbox", e))?;
    let total = outbox::count_entries(&pool)
        .await
        .map_err(|e| internal_err("Failed to count the notification outbox", e))?;
    Ok(Json(PaginatedResponse {
        data: entries
            .into_iter()
            .map(NotificationOutboxEntryResponse::from)
            .collect(),
        page: params.page(),
        per_page: params.per_page(),
        total,
    }))
}

#[utoipa::path(
    delete,
    path = "/notification-outbox/{id}",
    params(("id" = i64, Path, description = "Outbox entry ID")),
    responses(
        (status = 204, description = "Notification cancelled"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Notification not found, or already sent", body = ApiError),
        (status = 409, description = "Notification has not failed, or is being sent right now", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/notification-outbox",
    security(("session_cookie" = [])),
)]
pub async fn cancel_notification_outbox_entry(
    RequireAdmin(admin): RequireAdmin,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiErrorResponse> {
    let outcome = state
        .notifier
        .cancel_failing_outbox_entry(&state.pool, id)
        .await
        .map_err(|e| internal_err("Failed to cancel the notification", e))?;
    match outcome {
        CancelOutcome::Cancelled(cancelled) => {
            log_action_by_user!(admin, "Notification cancelled", outbox_id = id, notification = %cancelled.summary.as_deref().unwrap_or("unreadable"), channel = %cancelled.provider.as_deref().unwrap_or("unknown"), team_id = %id_or_none(cancelled.team_id), team_name = %cancelled.team.as_deref().unwrap_or("none"));
            Ok(StatusCode::NO_CONTENT)
        }
        CancelOutcome::BeingSent => Err(ApiErrorResponse::conflict(
            "This notification is being sent right now. If it fails, you can cancel it afterwards",
        )),
        CancelOutcome::NotFailing => Err(ApiErrorResponse::conflict(
            "This notification has not failed, so it cannot be cancelled",
        )),
        CancelOutcome::NotFound => Err(ApiErrorResponse::not_found("Notification")),
    }
}
