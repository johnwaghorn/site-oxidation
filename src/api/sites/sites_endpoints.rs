use super::{ApiError, ExpectedText, OutageResponse, SitePayload, SiteResponse};
use crate::api::errors::ApiErrorResponse;
use crate::api::pagination::{PaginatedResponse, PaginationParams};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::status::StatusCode,
};
use sqlx::SqlitePool;

#[utoipa::path(
    get,
    path = "/sites",
    params(PaginationParams),
    responses(
          (status = 200, description = "List all sites", body = inline(PaginatedResponse<SiteResponse>)),
          (status = 400, description = "Invalid query parameters", body = ApiError),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "sites",
    security(("bearer_auth" = []))
)]
pub async fn list_sites(
    State(pool): State<SqlitePool>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<SiteResponse>>, ApiErrorResponse> {
    let sites = sqlx::query_as::<_, SiteResponse>("SELECT id, name, url, expected_status, expected_text, status, last_checked_at, last_response_time_ms FROM sites LIMIT ? OFFSET ?")
        .bind(params.per_page())
        .bind(params.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch sites from database: {}", e);
            ApiErrorResponse::internal("Failed to fetch sites")
        })?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sites")
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site total from database: {}", e);
            ApiErrorResponse::internal("Failed to get site count")
        })?;
    let response = PaginatedResponse {
        data: sites,
        page: params.page(),
        per_page: params.per_page(),
        total,
    };
    Ok(Json(response))
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
    security(("bearer_auth" = []))
)]
pub async fn get_site(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<SiteResponse>, ApiErrorResponse> {
    sqlx::query_as::<_, SiteResponse>("SELECT id, name, url, expected_status, expected_text, status, last_checked_at, last_response_time_ms FROM sites WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch site from database: {}", e);
            ApiErrorResponse::internal("Failed to fetch site")
        })?
        .ok_or_else(|| ApiErrorResponse::not_found("Site"))
        .map(Json)
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
    security(("bearer_auth" = []))
)]
pub async fn get_site_outages(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<OutageResponse>>, ApiErrorResponse> {
    sqlx::query_scalar::<_, i64>("SELECT id FROM sites WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch site from database: {}", e);
            ApiErrorResponse::internal("Failed to fetch site")
        })?
        .ok_or_else(|| ApiErrorResponse::not_found("Site"))?;
    let outages = sqlx::query_as::<_, OutageResponse>("SELECT id, site_id, http_status, started_at, ended_at, error_message FROM outages WHERE site_id = ? LIMIT ? OFFSET ?")
        .bind(id)
        .bind(params.per_page())
        .bind(params.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch site outages from database: {}", e);
            ApiErrorResponse::internal("Failed to fetch outages")
        })?;
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages WHERE site_id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch total for outages from database: {}", e);
            ApiErrorResponse::internal("Failed to get outage count")
        })?;
    let response = PaginatedResponse {
        data: outages,
        page: params.page(),
        per_page: params.per_page(),
        total,
    };
    Ok(Json(response))
}

#[utoipa::path(
      post,
      path = "/sites",
      request_body = SitePayload,
      responses(
          (status = 201, description = "Site created", body = SiteResponse),
          (status = 401, description = "Unauthorized", body = ApiError),
          (status = 422, description = "Site payload validation error", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
      ),
      tag = "sites",
      security(("bearer_auth" = []))
)]
pub async fn create_site(
    State(pool): State<SqlitePool>,
    Json(payload): Json<SitePayload>,
) -> Result<(StatusCode, Json<SiteResponse>), ApiErrorResponse> {
    let result = sqlx::query_as::<_, SiteResponse>("INSERT INTO sites (name, url, expected_status, expected_text) VALUES (?, ?, ?, ?) RETURNING id, name, url, expected_status, expected_text, status, last_checked_at, last_response_time_ms")
        .bind(payload.name.as_str())
        .bind(payload.url.as_str())
        .bind(payload.expected_status.as_i64())
        .bind(payload.expected_text.as_ref().map(ExpectedText::as_str))
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create site: {}", e);
            ApiErrorResponse::internal("Failed to create site")
        })?;
    Ok((StatusCode::CREATED, Json(result)))
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
          (status = 422, description = "Site payload validation error", body = ApiError),
          (status = 500, description = "Internal server error", body = ApiError)
      ),
      tag = "sites",
      security(("bearer_auth" = []))
)]
pub async fn update_site(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<SitePayload>,
) -> Result<Json<SiteResponse>, ApiErrorResponse> {
    sqlx::query_as::<_, SiteResponse>("UPDATE sites SET name=?, url=?, expected_status=?, expected_text=? WHERE id = ? RETURNING id, name, url, expected_status, expected_text, status, last_checked_at, last_response_time_ms")
        .bind(payload.name.as_str())
        .bind(payload.url.as_str())
        .bind(payload.expected_status.as_i64())
        .bind(payload.expected_text.as_ref().map(ExpectedText::as_str))
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update site: {}", e);
            ApiErrorResponse::internal("Failed to update site")
        })?
        .ok_or_else(|| ApiErrorResponse::not_found("Site"))
        .map(Json)
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
    security(("bearer_auth" = []))
)]
pub async fn delete_site(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiErrorResponse> {
    // Note: `ON DELETE CASCADE` is set in the migration of the outages table,
    // therefore the related outages will be deleted too.
    let result = sqlx::query("DELETE FROM sites WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete site: {}", e);
            ApiErrorResponse::internal("Failed to delete site")
        })?;
    if result.rows_affected() == 0 {
        return Err(ApiErrorResponse::not_found("Site"));
    }
    Ok(StatusCode::NO_CONTENT)
}
