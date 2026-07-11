use super::queries;
use crate::api::errors::{ApiErrorResponse, internal_err};
use sqlx::SqlitePool;

pub async fn ensure_team_has_no_sites(
    pool: &SqlitePool,
    team_id: i64,
) -> Result<(), ApiErrorResponse> {
    let site_count: i64 = sqlx::query_scalar(queries::COUNT_TEAM_SITES)
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err("Failed to check team sites", e))?;
    if site_count > 0 {
        return Err(ApiErrorResponse::conflict(
            "Cannot delete team with assigned sites",
        ));
    }
    Ok(())
}

pub async fn ensure_team_deleted(
    pool: &SqlitePool,
    team_id: i64,
    deleted_team_id: Option<i64>,
) -> Result<(), ApiErrorResponse> {
    if deleted_team_id.is_some() {
        return Ok(());
    }
    let team_exists: i64 = sqlx::query_scalar(queries::TEAM_EXISTS)
        .bind(team_id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err("Failed to check team", e))?;
    if team_exists > 0 {
        return Err(ApiErrorResponse::conflict(
            "Cannot delete the last team assigned to a non-admin user",
        ));
    }
    Err(ApiErrorResponse::not_found("Team"))
}

pub async fn ensure_membership_removed(
    pool: &SqlitePool,
    team_id: i64,
    user_id: i64,
    removed_user_id: Option<i64>,
) -> Result<(), ApiErrorResponse> {
    if removed_user_id.is_some() {
        return Ok(());
    }
    let membership_exists: bool = sqlx::query_scalar(queries::MEMBERSHIP_EXISTS)
        .bind(team_id)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err("Failed to check team membership", e))?;
    if membership_exists {
        return Err(ApiErrorResponse::conflict(
            "Cannot remove a non-admin user's last team",
        ));
    }
    Err(ApiErrorResponse::not_found("Membership"))
}
