use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::auth_backend::AuthSession;
use crate::models::user::{User, UserRole};

/// Request body extractor that returns rejections in the documented `ApiError` envelope.
pub struct JsonPayload<T>(pub T);

impl<S, T> FromRequest<S> for JsonPayload<T>
where
    S: Send + Sync,
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiErrorResponse;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(JsonPayload(value)),
            Err(JsonRejection::JsonDataError(rejection)) => {
                Err(ApiErrorResponse::validation(&rejection.body_text()))
            }
            Err(rejection) => Err(ApiErrorResponse::from_status(
                rejection.status(),
                &rejection.body_text(),
            )),
        }
    }
}

/// JSON payload whose validation is deferred to the handler, so
/// pre-validation work (e.g. rate-limit accounting) runs even for invalid
/// bodies. Call `validated()` to obtain the payload or the rejection.
pub struct DeferredJsonPayload<T>(Result<JsonPayload<T>, ApiErrorResponse>);

impl<T> DeferredJsonPayload<T> {
    pub fn validated(self) -> Result<T, ApiErrorResponse> {
        self.0.map(|JsonPayload(payload)| payload)
    }
}

impl<S, T> FromRequest<S> for DeferredJsonPayload<T>
where
    S: Send + Sync,
    JsonPayload<T>: FromRequest<S, Rejection = ApiErrorResponse>,
{
    type Rejection = Infallible;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Ok(DeferredJsonPayload(
            JsonPayload::from_request(req, state).await,
        ))
    }
}

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
