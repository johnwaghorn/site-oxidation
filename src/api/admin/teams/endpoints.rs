use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use sqlx::SqlitePool;

use super::queries;
use super::requests::{AddMemberRequest, CreateTeamRequest, UpdateTeamRequest};
use super::responses::{TeamOption, TeamResponse};
use crate::api::admin::responses::SuccessResponse;
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err, unique_conflict_err};
use crate::api::extractors::RequireAdmin;
use crate::api::pagination::{PaginatedResponse, PaginationParams, deserialize_u32_params};
use crate::api::search::{SearchParams, normalize_search};
use crate::api::sites::responses::SiteResponse;

#[derive(Deserialize)]
pub struct ListTeamsQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    per_page: Option<u32>,
    search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/teams",
    params(PaginationParams, SearchParams),
    responses(
        (status = 200, description = "List all teams", body = inline(PaginatedResponse<TeamResponse>)),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn list_teams(
    RequireAdmin(_user): RequireAdmin,
    State(pool): State<SqlitePool>,
    Query(params): Query<ListTeamsQuery>,
) -> Result<Json<PaginatedResponse<TeamResponse>>, ApiErrorResponse> {
    let pagination = PaginationParams::new(params.page, params.per_page);
    let search = normalize_search(params.search.as_deref());
    let teams = sqlx::query_as::<_, TeamResponse>(queries::LIST_TEAMS)
        .bind(search)
        .bind(pagination.per_page())
        .bind(pagination.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list teams", e))?;
    let total: i64 = sqlx::query_scalar(queries::COUNT_TEAMS)
        .bind(search)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to count teams", e))?;
    Ok(Json(PaginatedResponse {
        data: teams,
        page: pagination.page(),
        per_page: pagination.per_page(),
        total,
    }))
}

#[utoipa::path(
    get,
    path = "/teams/{id}",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 200, description = "Team details", body = TeamResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn get_team(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<TeamResponse>, ApiErrorResponse> {
    let team = sqlx::query_as::<_, TeamResponse>(queries::SELECT_TEAM)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to get team", e))?
        .ok_or_else(|| ApiErrorResponse::not_found("Team"))?;
    Ok(Json(team))
}

#[utoipa::path(
    get,
    path = "/teams/{id}/sites",
    params(
        ("id" = i64, Path, description = "Team ID"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "List sites assigned to a team", body = inline(PaginatedResponse<SiteResponse>)),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn list_team_sites(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<SiteResponse>>, ApiErrorResponse> {
    let team_exists: i64 = sqlx::query_scalar(queries::TEAM_EXISTS)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check team", e))?;
    if team_exists == 0 {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    let sites = sqlx::query_as::<_, SiteResponse>(queries::LIST_TEAM_SITES)
        .bind(id)
        .bind(params.per_page())
        .bind(params.offset())
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list team sites", e))?;
    let total: i64 = sqlx::query_scalar(queries::COUNT_TEAM_SITES)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to count team sites", e))?;
    Ok(Json(PaginatedResponse {
        data: sites,
        page: params.page(),
        per_page: params.per_page(),
        total,
    }))
}

#[utoipa::path(
    delete,
    path = "/teams/{id}/sites/{site_id}",
    params(
        ("id" = i64, Path, description = "Team ID"),
        ("site_id" = i64, Path, description = "Site ID"),
    ),
    responses(
        (status = 204, description = "Site removed from team"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Site is not assigned to team", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn unassign_team_site(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path((team_id, site_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiErrorResponse> {
    let updated: Option<i64> = sqlx::query_scalar(queries::UNASSIGN_TEAM_SITE)
        .bind(team_id)
        .bind(site_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to remove site from team", e))?;
    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("Site assignment"));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/teams/options",
    params(SearchParams),
    responses(
        (status = 200, description = "Up to 20 matching teams (id/name) for a selector typeahead", body = [TeamOption]),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn list_team_options(
    RequireAdmin(_user): RequireAdmin,
    State(pool): State<SqlitePool>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<TeamOption>>, ApiErrorResponse> {
    const LIMIT: i64 = 20;
    let options = sqlx::query_as::<_, TeamOption>(queries::SEARCH_TEAM_OPTIONS)
        .bind(params.normalized())
        .bind(LIMIT)
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list team options", e))?;
    Ok(Json(options))
}

#[utoipa::path(
    post,
    path = "/teams",
    request_body = CreateTeamRequest,
    responses(
        (status = 201, description = "Team created", body = TeamResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 409, description = "Team name already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn create_team(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<TeamResponse>), ApiErrorResponse> {
    let name = payload.name.trim().to_owned();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ApiErrorResponse::validation(
            "Team name must be between 1 and 60 characters",
        ));
    }
    let id: i64 = sqlx::query_scalar(queries::INSERT_TEAM)
        .bind(&name)
        .fetch_one(&pool)
        .await
        .map_err(|e| unique_conflict_err("Team name already exists", "Failed to create team", e))?;
    Ok((
        StatusCode::CREATED,
        Json(TeamResponse {
            id,
            name,
            member_count: 0,
            site_count: 0,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/teams/{id}",
    params(("id" = i64, Path, description = "Team ID")),
    request_body = UpdateTeamRequest,
    responses(
        (status = 200, description = "Team renamed", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 409, description = "Team name already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn update_team(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTeamRequest>,
) -> Result<Json<SuccessResponse>, ApiErrorResponse> {
    let name = payload.name.trim().to_owned();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(ApiErrorResponse::validation(
            "Team name must be between 1 and 60 characters",
        ));
    }
    let updated: Option<i64> = sqlx::query_scalar(queries::UPDATE_TEAM)
        .bind(&name)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| unique_conflict_err("Team name already exists", "Failed to rename team", e))?;

    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    delete,
    path = "/teams/{id}",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 204, description = "Team deleted"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 409, description = "Team has assigned sites or is the last team for a non-admin member", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn delete_team(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiErrorResponse> {
    let site_count: i64 = sqlx::query_scalar(queries::COUNT_TEAM_SITES)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check team sites", e))?;
    if site_count > 0 {
        return Err(ApiErrorResponse::conflict(
            "Cannot delete team with assigned sites",
        ));
    }
    let deleted: Option<i64> = sqlx::query_scalar(queries::DELETE_TEAM)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to delete team", e))?;
    if deleted.is_none() {
        let team_exists: i64 = sqlx::query_scalar(queries::TEAM_EXISTS)
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| internal_err("Failed to check team", e))?;
        if team_exists > 0 {
            return Err(ApiErrorResponse::conflict(
                "Cannot delete the last team assigned to a non-admin user",
            ));
        }
        return Err(ApiErrorResponse::not_found("Team"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/teams/{id}/members",
    params(("id" = i64, Path, description = "Team ID")),
    request_body = AddMemberRequest,
    responses(
        (status = 201, description = "Member added", body = SuccessResponse),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team or user not found", body = ApiError),
        (status = 409, description = "User already a member", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn add_team_member(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path(team_id): Path<i64>,
    Json(payload): Json<AddMemberRequest>,
) -> Result<(StatusCode, Json<SuccessResponse>), ApiErrorResponse> {
    let team_exists: i64 = sqlx::query_scalar(queries::TEAM_EXISTS)
        .bind(team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check team", e))?;
    if team_exists == 0 {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    let user_exists: i64 = sqlx::query_scalar(queries::USER_EXISTS)
        .bind(payload.user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check user", e))?;
    if user_exists == 0 {
        return Err(ApiErrorResponse::not_found("User"));
    }
    sqlx::query(queries::ADD_TEAM_MEMBER)
        .bind(team_id)
        .bind(payload.user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            unique_conflict_err(
                "User is already a member of this team",
                "Failed to add team member",
                e,
            )
        })?;
    Ok((StatusCode::CREATED, Json(SuccessResponse { success: true })))
}

#[utoipa::path(
    delete,
    path = "/teams/{id}/members/{user_id}",
    params(
        ("id" = i64, Path, description = "Team ID"),
        ("user_id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Membership not found", body = ApiError),
        (status = 409, description = "Cannot remove a non-admin user's last team", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "admin/teams",
    security(("session_cookie" = [])),
)]
pub async fn remove_team_member(
    RequireAdmin(_admin): RequireAdmin,
    State(pool): State<SqlitePool>,
    Path((team_id, user_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiErrorResponse> {
    let deleted: Option<i64> = sqlx::query_scalar(queries::REMOVE_TEAM_MEMBER)
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to remove team member", e))?;

    if deleted.is_none() {
        let membership_exists: bool = sqlx::query_scalar(queries::MEMBERSHIP_EXISTS)
            .bind(team_id)
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| internal_err("Failed to check team membership", e))?;
        if membership_exists {
            return Err(ApiErrorResponse::conflict(
                "Cannot remove a non-admin user's last team",
            ));
        }
        return Err(ApiErrorResponse::not_found("Membership"));
    }
    Ok(StatusCode::NO_CONTENT)
}
