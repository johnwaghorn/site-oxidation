use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::SqlitePool;

use super::{
    AddMemberRequest, CreateTeamRequest, SuccessResponse, TeamResponse, UpdateTeamRequest,
    admin_queries,
};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::api::extractors::RequireAdmin;

#[utoipa::path(
    get,
    path = "/admin/teams",
    responses(
        (status = 200, description = "List all teams", body = Vec<TeamResponse>),
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
) -> Result<Json<Vec<TeamResponse>>, ApiErrorResponse> {
    let teams = sqlx::query_as::<_, TeamResponse>(admin_queries::LIST_TEAMS)
        .fetch_all(&pool)
        .await
        .map_err(|e| internal_err("Failed to list teams", e))?;
    Ok(Json(teams))
}

#[utoipa::path(
    post,
    path = "/admin/teams",
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
    if name.is_empty() || name.len() > 100 {
        return Err(ApiErrorResponse::validation(
            "Team name must be between 1 and 100 characters",
        ));
    }
    let id: i64 = sqlx::query_scalar(admin_queries::INSERT_TEAM)
        .bind(&name)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                ApiErrorResponse::conflict("Team name already exists")
            } else {
                internal_err("Failed to create team", e)
            }
        })?;
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
    path = "/admin/teams/{id}",
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
    if name.is_empty() || name.len() > 100 {
        return Err(ApiErrorResponse::validation(
            "Team name must be between 1 and 100 characters",
        ));
    }
    let updated: Option<i64> = sqlx::query_scalar(admin_queries::UPDATE_TEAM)
        .bind(&name)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                ApiErrorResponse::conflict("Team name already exists")
            } else {
                internal_err("Failed to rename team", e)
            }
        })?;

    if updated.is_none() {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    Ok(Json(SuccessResponse { success: true }))
}

#[utoipa::path(
    delete,
    path = "/admin/teams/{id}",
    params(("id" = i64, Path, description = "Team ID")),
    responses(
        (status = 204, description = "Team deleted"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Team not found", body = ApiError),
        (status = 409, description = "Team has assigned sites", body = ApiError),
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
    let site_count: i64 = sqlx::query_scalar(admin_queries::COUNT_TEAM_SITES)
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check team sites", e))?;
    if site_count > 0 {
        return Err(ApiErrorResponse::conflict(
            "Cannot delete team with assigned sites",
        ));
    }
    let deleted: Option<i64> = sqlx::query_scalar(admin_queries::DELETE_TEAM)
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to delete team", e))?;
    if deleted.is_none() {
        return Err(ApiErrorResponse::not_found("Team"));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/admin/teams/{id}/members",
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
    let team_exists: i64 = sqlx::query_scalar(admin_queries::TEAM_EXISTS)
        .bind(team_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check team", e))?;
    if team_exists == 0 {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    let user_exists: i64 = sqlx::query_scalar(admin_queries::USER_EXISTS)
        .bind(payload.user_id)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check user", e))?;
    if user_exists == 0 {
        return Err(ApiErrorResponse::not_found("User"));
    }
    sqlx::query(admin_queries::ADD_TEAM_MEMBER)
        .bind(team_id)
        .bind(payload.user_id)
        .execute(&pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed")
                || e.to_string().contains("PRIMARY KEY constraint failed")
            {
                ApiErrorResponse::conflict("User is already a member of this team")
            } else {
                internal_err("Failed to add team member", e)
            }
        })?;
    Ok((StatusCode::CREATED, Json(SuccessResponse { success: true })))
}

#[utoipa::path(
    delete,
    path = "/admin/teams/{id}/members/{user_id}",
    params(
        ("id" = i64, Path, description = "Team ID"),
        ("user_id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "Member removed"),
        (status = 401, description = "Unauthorized", body = ApiError),
        (status = 403, description = "Admin access required", body = ApiError),
        (status = 404, description = "Membership not found", body = ApiError),
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
    let deleted: Option<i64> = sqlx::query_scalar(admin_queries::REMOVE_TEAM_MEMBER)
        .bind(team_id)
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| internal_err("Failed to remove team member", e))?;

    if deleted.is_none() {
        return Err(ApiErrorResponse::not_found("Membership"));
    }
    Ok(StatusCode::NO_CONTENT)
}
