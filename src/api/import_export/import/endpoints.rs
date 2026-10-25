use axum::Json;
use axum::extract::{Query, State};

use super::queries::IMPORT_SITE;
use super::requests::{ImportSitesParams, SiteImportRequest};
use super::responses::{ImportOutcome, ImportedSiteResult, SiteImportResponse};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::{JsonPayload, RequireAppAccess};
use crate::api::import_export::SITE_EXPORT_VERSION;
use crate::api::sites::access::resolve_team_id;
use crate::api::sites::fields::ExpectedText;
use crate::api::sites::rules::ensure_site_url_allowed;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/import",
    params(ImportSitesParams),
    request_body = SiteImportRequest,
    responses(
        (status = 200, description = "What happened to each site in the file, in file order", body = SiteImportResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Not a member of the team", body = ApiError),
        (status = 422, description = "Invalid file; nothing was imported", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "import-export",
    security(("session_cookie" = []))
)]
pub async fn import_sites(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Query(params): Query<ImportSitesParams>,
    JsonPayload(file): JsonPayload<SiteImportRequest>,
) -> Result<Json<SiteImportResponse>, ApiErrorResponse> {
    let team_id = resolve_team_id(&state.pool, &user, params.team_id).await?;
    if file.version != SITE_EXPORT_VERSION {
        return Err(ApiErrorResponse::validation(&format!(
            "Unsupported export version {}, this instance reads version {SITE_EXPORT_VERSION}",
            file.version
        )));
    }
    for (index, site) in file.sites.iter().enumerate() {
        ensure_site_url_allowed(&site.url, state.config.probe_allow_private_ips).map_err(|_| {
            ApiErrorResponse::validation(&format!(
                "sites[{index}].url: Private/internal IP addresses are not allowed"
            ))
        })?;
    }
    let mut transaction = state
        .pool
        .begin()
        .await
        .map_err(|e| internal_err("Failed to start site import", e))?;
    let mut results = Vec::new();
    for site in &file.sites {
        let inserted = sqlx::query(IMPORT_SITE)
            .bind(site.name.as_str())
            .bind(site.url.as_str())
            .bind(site.expected_status.as_i64())
            .bind(site.expected_text.as_ref().map(ExpectedText::as_str))
            .bind(site.probe_interval_seconds.as_i64())
            .bind(team_id)
            .bind(site.tls_allow_untrusted)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                internal_err(
                    &format!("Failed to import site '{}'", site.name.as_str()),
                    e,
                )
            })?
            .rows_affected()
            > 0;
        results.push(ImportedSiteResult {
            name: site.name.as_str().to_owned(),
            url: site.url.as_str().to_owned(),
            outcome: if inserted {
                ImportOutcome::Added
            } else {
                ImportOutcome::Skipped
            },
        });
    }
    transaction
        .commit()
        .await
        .map_err(|e| internal_err("Failed to commit site import", e))?;
    Ok(Json(SiteImportResponse { sites: results }))
}
