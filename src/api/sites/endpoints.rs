use super::access::{ensure_site_access, resolve_team_id};
use super::fields::ExpectedText;
use super::queries::{
    COUNT_OUTAGES, COUNT_SITES_ADMIN, COUNT_SITES_USER, DELETE_SITE, INSERT_SITE, LIST_OUTAGES,
    LIST_SITES_ADMIN, LIST_SITES_USER, SELECT_SITE_ADMIN, SELECT_SITE_USER, UPDATE_SITE,
};
use super::rules;
use super::{OutageResponse, SitePayload, SiteResponse};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err, unique_conflict_err};
use crate::api::extractors::{JsonPayload, RequireAppAccess};
use crate::api::pagination::{PaginatedResponse, PaginationParams, deserialize_u32_params};
use crate::api::search::{SearchParams, normalise_search};
use crate::models::user::UserRole;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::status::StatusCode,
};

async fn reload_site(pool: &sqlx::SqlitePool, id: i64) -> Result<SiteResponse, ApiErrorResponse> {
    sqlx::query_as::<_, SiteResponse>(SELECT_SITE_ADMIN)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err(&format!("Failed to reload site {id}"), e))
}

#[utoipa::path(
    get,
    path = "/sites",
    params(PaginationParams, SearchParams),
    responses(
          (status = 200, description = "List all sites", body = inline(PaginatedResponse<SiteResponse>)),
          (status = 400, description = "Invalid query parameters", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "sites",
    security(("session_cookie" = []))
)]
pub async fn list_sites(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Query(params): Query<ListSitesQuery>,
) -> Result<Json<PaginatedResponse<SiteResponse>>, ApiErrorResponse> {
    let pagination = PaginationParams::new(params.page, params.per_page);
    let search = normalise_search(params.search.as_deref());
    let (sites, total) = if user.role == UserRole::Admin {
        let sites = sqlx::query_as::<_, SiteResponse>(LIST_SITES_ADMIN)
            .bind(search)
            .bind(pagination.per_page())
            .bind(pagination.offset())
            .fetch_all(&state.pool)
            .await
            .map_err(|e| internal_err("Failed to list all sites", e))?;
        let total: i64 = sqlx::query_scalar(COUNT_SITES_ADMIN)
            .bind(search)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| internal_err("Failed to count all sites", e))?;
        (sites, total)
    } else {
        let sites = sqlx::query_as::<_, SiteResponse>(LIST_SITES_USER)
            .bind(user.id)
            .bind(search)
            .bind(pagination.per_page())
            .bind(pagination.offset())
            .fetch_all(&state.pool)
            .await
            .map_err(|e| internal_err(&format!("Failed to list sites for user {}", user.id), e))?;
        let total: i64 = sqlx::query_scalar(COUNT_SITES_USER)
            .bind(user.id)
            .bind(search)
            .fetch_one(&state.pool)
            .await
            .map_err(|e| internal_err(&format!("Failed to count sites for user {}", user.id), e))?;
        (sites, total)
    };
    Ok(Json(PaginatedResponse {
        data: sites,
        page: pagination.page(),
        per_page: pagination.per_page(),
        total,
    }))
}

#[derive(serde::Deserialize)]
pub struct ListSitesQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    per_page: Option<u32>,
    search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/sites/{id}",
    params(
          ("id" = i64, Path, description = "Site ID")
    ),
    responses(
          (status = 200, description = "Site found", body = SiteResponse),
          (status = 400, description = "Invalid site ID", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 404, description = "Site not found", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "sites",
    security(("session_cookie" = []))
)]
pub async fn get_site(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SiteResponse>, ApiErrorResponse> {
    let site = if user.role == UserRole::Admin {
        sqlx::query_as::<_, SiteResponse>(SELECT_SITE_ADMIN)
            .bind(id)
            .fetch_optional(&state.pool)
            .await
    } else {
        sqlx::query_as::<_, SiteResponse>(SELECT_SITE_USER)
            .bind(id)
            .bind(user.id)
            .fetch_optional(&state.pool)
            .await
    }
    .map_err(|e| {
        internal_err(
            &format!("Failed to fetch site {id} for user {}", user.id),
            e,
        )
    })?
    .ok_or_else(|| ApiErrorResponse::not_found("Site"))?;

    Ok(Json(site))
}

#[utoipa::path(
    get,
    path = "/sites/{id}/outages",
    params(
        PaginationParams,
        ("id" = i64, Path, description = "Site ID")
    ),
    responses(
          (status = 200, description = "List of outages", body = inline(PaginatedResponse<OutageResponse>)),
          (status = 400, description = "Invalid site ID or query parameters", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 404, description = "Site not found", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "sites",
    security(("session_cookie" = []))
)]
pub async fn get_site_outages(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<OutageResponse>>, ApiErrorResponse> {
    ensure_site_access(&state.pool, id, &user).await?;
    let outages = sqlx::query_as::<_, OutageResponse>(LIST_OUTAGES)
        .bind(id)
        .bind(params.per_page())
        .bind(params.offset())
        .fetch_all(&state.pool)
        .await
        .map_err(|e| internal_err(&format!("Failed to list outages for site {id}"), e))?;
    let total: i64 = sqlx::query_scalar(COUNT_OUTAGES)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| internal_err(&format!("Failed to count outages for site {id}"), e))?;
    Ok(Json(PaginatedResponse {
        data: outages,
        page: params.page(),
        per_page: params.per_page(),
        total,
    }))
}

#[utoipa::path(
      post,
      path = "/sites",
      request_body = SitePayload,
      responses(
          (status = 201, description = "Site created", body = SiteResponse),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 409, description = "A site with that URL already exists for this team", body = ApiError),
          (status = 422, description = "Site payload validation error", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
      ),
      tag = "sites",
      security(("session_cookie" = []))
)]
pub async fn create_site(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    JsonPayload(payload): JsonPayload<SitePayload>,
) -> Result<(StatusCode, Json<SiteResponse>), ApiErrorResponse> {
    let team_id = resolve_team_id(&state.pool, &user, payload.team_id).await?;
    rules::ensure_site_url_allowed(&payload.url, state.config.probe_allow_private_ips)?;
    let id: i64 = sqlx::query_scalar(INSERT_SITE)
        .bind(payload.name.as_str())
        .bind(payload.url.as_str())
        .bind(payload.expected_status.as_i64())
        .bind(payload.expected_text.as_ref().map(ExpectedText::as_str))
        .bind(payload.probe_interval_seconds.as_i64())
        .bind(team_id)
        .bind(payload.tls_allow_untrusted)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            unique_conflict_err(
                "A site with that URL already exists for this team",
                &format!("Failed to insert site '{}'", payload.name.as_str()),
                e,
            )
        })?;
    let site = reload_site(&state.pool, id).await?;
    Ok((StatusCode::CREATED, Json(site)))
}

#[utoipa::path(
      put,
      path = "/sites/{id}",
      params(("id" = i64, Path, description = "Site ID")),
      request_body = SitePayload,
      responses(
          (status = 200, description = "Site updated", body = SiteResponse),
          (status = 400, description = "Invalid Site ID", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 404, description = "Site not found", body = ApiError),
          (status = 409, description = "A site with that URL already exists for this team", body = ApiError),
          (status = 422, description = "Site payload validation error", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
      ),
      tag = "sites",
      security(("session_cookie" = []))
)]
pub async fn update_site(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    JsonPayload(payload): JsonPayload<SitePayload>,
) -> Result<Json<SiteResponse>, ApiErrorResponse> {
    ensure_site_access(&state.pool, id, &user).await?;
    let team_id = resolve_team_id(&state.pool, &user, payload.team_id).await?;
    rules::ensure_site_url_allowed(&payload.url, state.config.probe_allow_private_ips)?;
    let updated: Option<i64> = sqlx::query_scalar(UPDATE_SITE)
        .bind(payload.name.as_str())
        .bind(payload.url.as_str())
        .bind(payload.expected_status.as_i64())
        .bind(payload.expected_text.as_ref().map(ExpectedText::as_str))
        .bind(payload.probe_interval_seconds.as_i64())
        .bind(team_id)
        .bind(payload.tls_allow_untrusted)
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            unique_conflict_err(
                "A site with that URL already exists for this team",
                &format!("Failed to update site {id}"),
                e,
            )
        })?;
    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("Site"));
    }
    let site = reload_site(&state.pool, id).await?;
    Ok(Json(site))
}

#[utoipa::path(
    delete,
    path = "/sites/{id}",
    params(
          ("id" = i64, Path, description = "Site ID")
    ),
    responses(
          (status = 204, description = "Site deleted"),
          (status = 400, description = "Invalid site ID", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 404, description = "Site not found", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "sites",
    security(("session_cookie" = []))
)]
pub async fn delete_site(
    RequireAppAccess(user): RequireAppAccess,
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiErrorResponse> {
    ensure_site_access(&state.pool, id, &user).await?;
    // Note: `ON DELETE CASCADE` is set in the migration of the outages table,
    // therefore the related outages will be deleted too.
    let result = sqlx::query(DELETE_SITE)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| internal_err(&format!("Failed to delete site {id}"), e))?;
    if result.rows_affected() == 0 {
        return Err(ApiErrorResponse::not_found("Site"));
    }
    Ok(StatusCode::NO_CONTENT)
}
