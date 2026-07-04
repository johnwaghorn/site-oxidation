use super::queries::{CHECK_SITE_ACCESS_ADMIN, CHECK_SITE_ACCESS_USER};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::api::teams::access::ensure_team_access;
use crate::models::user::{User, UserRole};

use sqlx::SqlitePool;

pub async fn ensure_site_access(
    pool: &SqlitePool,
    site_id: i64,
    user: &User,
) -> Result<(), ApiErrorResponse> {
    let can_access: bool = if user.role == UserRole::Admin {
        sqlx::query_scalar(CHECK_SITE_ACCESS_ADMIN)
            .bind(site_id)
            .fetch_one(pool)
            .await
    } else {
        sqlx::query_scalar(CHECK_SITE_ACCESS_USER)
            .bind(site_id)
            .bind(user.id)
            .fetch_one(pool)
            .await
    }
    .map_err(|e| {
        internal_err(
            &format!("Failed access check for site {site_id}, user {}", user.id),
            e,
        )
    })?;
    if !can_access {
        return Err(ApiErrorResponse::not_found("Site"));
    }
    Ok(())
}

pub async fn resolve_team_id(
    pool: &SqlitePool,
    user: &User,
    team_id: Option<i64>,
) -> Result<Option<i64>, ApiErrorResponse> {
    match (user.role, team_id) {
        (UserRole::Admin, team_id) => Ok(team_id),
        (UserRole::User, Some(id)) => {
            ensure_team_access(pool, id, user).await?;
            Ok(Some(id))
        }
        (UserRole::User, None) => Err(ApiErrorResponse::validation("team_id is required")),
    }
}
