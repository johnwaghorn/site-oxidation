use super::queries::{CHECK_TEAM_MEMBERSHIP, TEAM_EXISTS};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::models::user::{User, UserRole};
use crate::state::AppState;
use sqlx::SqlitePool;

pub async fn ensure_team_access(
    pool: &SqlitePool,
    team_id: i64,
    user: &User,
) -> Result<(), ApiErrorResponse> {
    if user.role == UserRole::Admin {
        return Ok(());
    }
    let is_member: bool = sqlx::query_scalar(CHECK_TEAM_MEMBERSHIP)
        .bind(team_id)
        .bind(user.id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            internal_err(
                &format!("Failed access check for team {team_id}, user {}", user.id),
                e,
            )
        })?;
    if !is_member {
        return Err(ApiErrorResponse::forbidden(
            "You are not a member of this team",
        ));
    }
    Ok(())
}

pub async fn ensure_team_exists(state: &AppState, id: i64) -> Result<(), ApiErrorResponse> {
    let exists: bool = sqlx::query_scalar(TEAM_EXISTS)
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| internal_err("Failed to check team", e))?;
    if !exists {
        return Err(ApiErrorResponse::not_found("Team"));
    }
    Ok(())
}
