use axum::Json;

use super::responses::VersionResponse;
use crate::api::errors::ApiError;
use crate::api::extractors::RequireAuth;

#[utoipa::path(
    get,
    path = "/version",
    responses(
        (status = 200, description = "Running application version", body = VersionResponse),
        (status = 401, description = "Not authenticated", body = ApiError),
    ),
    tag = "version",
    security(("session_cookie" = [])),
)]
pub async fn version(RequireAuth(_user): RequireAuth) -> Json<VersionResponse> {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION"),
    })
}
