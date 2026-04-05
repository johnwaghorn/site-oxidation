use anyhow::{Context, Result};

use crate::config::AppConfig;
use tower_http::cors::CorsLayer;

pub fn cors_layer(config: &AppConfig) -> Result<CorsLayer> {
    match config.allowed_origin {
        Some(ref origin) => {
            let header_value = origin
                .parse::<axum::http::HeaderValue>()
                .with_context(|| format!("Invalid CORS_ALLOWED_ORIGIN: {origin}"))?;
            Ok(CorsLayer::new()
                .allow_origin(header_value)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::PATCH,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([axum::http::header::CONTENT_TYPE])
                .allow_credentials(true))
        }
        None => Ok(CorsLayer::new()),
    }
}
