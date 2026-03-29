use axum::{extract::FromRequestParts, http::request::Parts};

use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::auth_backend::AuthSession;
use crate::models::user::{User, UserRole};

pub struct RequireAuth(pub User);
pub struct RequireAppAccess(pub User);
pub struct RequireAdmin(pub User);

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = ApiErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_session = AuthSession::from_request_parts(parts, state)
            .await
            .map_err(|(status, msg)| {
                internal_err("Auth session extraction failed", format!("{status}: {msg}"))
            })?;

        match auth_session.user {
            Some(user) => Ok(RequireAuth(user)),
            None => Err(ApiErrorResponse::unauthorized()),
        }
    }
}

impl<S> FromRequestParts<S> for RequireAppAccess
where
    S: Send + Sync,
{
    type Rejection = ApiErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let RequireAuth(user) = RequireAuth::from_request_parts(parts, state).await?;
        if user.must_change_password {
            return Err(ApiErrorResponse::forbidden(
                "Password must be changed before accessing this resource",
            ));
        }
        Ok(RequireAppAccess(user))
    }
}

impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
{
    type Rejection = ApiErrorResponse;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let RequireAppAccess(user) = RequireAppAccess::from_request_parts(parts, state).await?;
        if user.role == UserRole::Admin {
            Ok(RequireAdmin(user))
        } else {
            Err(ApiErrorResponse::forbidden("Admin access required"))
        }
    }
}
