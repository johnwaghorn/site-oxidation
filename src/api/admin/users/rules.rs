use super::queries;
use super::requests::UpdateUserRequest;
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::models::user::{User, UserRole};
use sqlx::SqlitePool;

pub fn self_update_error(
    admin: &User,
    target_user_id: i64,
    payload: &UpdateUserRequest,
) -> Option<&'static str> {
    if target_user_id == admin.id && !payload.active {
        return Some("Cannot deactivate your own account");
    }
    if target_user_id == admin.id && payload.role != UserRole::Admin {
        return Some("Cannot demote your own account");
    }
    None
}

pub fn self_delete_error(admin: &User, target_user_id: i64) -> Option<&'static str> {
    if target_user_id == admin.id {
        return Some("Cannot delete your own account");
    }
    None
}

pub async fn resolve_team_for_role(
    pool: &SqlitePool,
    role: UserRole,
    team_id: Option<i64>,
) -> Result<Option<i64>, ApiErrorResponse> {
    match role {
        UserRole::Admin => Ok(None),
        UserRole::User => {
            let team_id = team_id.ok_or_else(|| {
                ApiErrorResponse::validation("A team is required for non-admin users")
            })?;
            let team_exists: i64 = sqlx::query_scalar(queries::TEAM_EXISTS)
                .bind(team_id)
                .fetch_one(pool)
                .await
                .map_err(|e| internal_err("Failed to check team", e))?;
            if team_exists == 0 {
                return Err(ApiErrorResponse::not_found("Team"));
            }
            Ok(Some(team_id))
        }
    }
}

pub async fn ensure_not_last_active_admin(
    pool: &SqlitePool,
    target_user_id: i64,
    payload: &UpdateUserRequest,
) -> Result<(), ApiErrorResponse> {
    if payload.role == UserRole::Admin && payload.active {
        return Ok(());
    }
    let active_admins: i64 = sqlx::query_scalar(queries::COUNT_ACTIVE_ADMINS)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err("Failed to count active admins", e))?;
    let is_target_active_admin: bool = sqlx::query_scalar(queries::IS_ACTIVE_ADMIN)
        .bind(target_user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| internal_err("Failed to check user status", e))?
        .unwrap_or(false);
    if is_target_active_admin && active_admins <= 1 {
        return Err(ApiErrorResponse::conflict(
            "Cannot demote or deactivate the last active admin",
        ));
    }
    Ok(())
}

pub async fn ensure_user_mutation_applied(
    pool: &SqlitePool,
    target_user_id: i64,
    mutated_user_id: Option<i64>,
    conflict_message: &'static str,
) -> Result<(), ApiErrorResponse> {
    if mutated_user_id.is_some() {
        return Ok(());
    }
    let user_exists: bool = sqlx::query_scalar(queries::USER_EXISTS)
        .bind(target_user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| internal_err("Failed to check user", e))?;
    if user_exists {
        return Err(ApiErrorResponse::conflict(conflict_message));
    }
    Err(ApiErrorResponse::not_found("User"))
}
