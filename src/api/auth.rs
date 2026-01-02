use crate::api::errors::ApiErrorResponse;
use crate::state::AppState;
use axum::extract::State;
use axum::{body::Body, http::Request, middleware::Next, response::Response};

pub async fn require_api_key(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ApiErrorResponse> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());
    match auth_header {
        Some(header) if header == format!("Bearer {}", state.config.api_key) => {
            Ok(next.run(request).await)
        }
        _ => Err(ApiErrorResponse::unauthorized()),
    }
}
