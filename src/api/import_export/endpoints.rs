use axum::Json;
use axum::extract::State;
use chrono::Utc;

use super::queries::{EXPORT_SITES_ADMIN, EXPORT_SITES_USER};
use super::responses::{ExportedSite, SiteExportResponse};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAppAccess;
use crate::models::user::UserRole;
use crate::state::AppState;

// Use so import can tell old files apart
const SITE_EXPORT_VERSION: u32 = 1;

#[utoipa::path(
    get,
    path = "/sites/export",
    responses(
        (status = 200, description = "Settings of every site the user can see, for import into another instance", body = SiteExportResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "import-export",
    security(("session_cookie" = []))
)]
pub async fn export_sites(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
) -> Result<Json<SiteExportResponse>, ApiErrorResponse> {
    let sites = if user.role == UserRole::Admin {
        sqlx::query_as::<_, ExportedSite>(EXPORT_SITES_ADMIN)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| internal_err("Failed to export all sites", e))?
    } else {
        sqlx::query_as::<_, ExportedSite>(EXPORT_SITES_USER)
            .bind(user.id)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| internal_err(&format!("Failed to export sites for user {}", user.id), e))?
    };
    Ok(Json(SiteExportResponse {
        version: SITE_EXPORT_VERSION,
        exported_at: Utc::now(),
        exported_by: env!("CARGO_PKG_VERSION"),
        sites,
    }))
}
