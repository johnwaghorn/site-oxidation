mod api;
mod config;
mod db;
mod jobs;
mod models;
mod state;
#[cfg(test)]
mod tests;

use api::ApiDoc;
use api::auth::require_api_key;
use api::health;
use axum::{Router, middleware, routing::get};
use config::AppConfig;
use jobs::check_all_sites;
use reqwest::Client;
use state::AppState;
use std::time::Duration;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = AppConfig::from_env();
    let port = config.server_port;
    let check_interval = config.check_interval_secs;
    let pool = db::init_db(&config.database_path)
        .await
        .expect("Could not initialize database");
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
    };
    let client = Client::builder()
        .user_agent("SiteOxidation/1.0 (+https://github.com/johnwaghorn/site-oxidation)")
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Failed to create HTTP client");
    let checker_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(check_interval));
        loop {
            interval.tick().await;
            check_all_sites(&client, &checker_pool).await;
        }
    });
    let static_service = ServeDir::new("static").fallback(ServeFile::new("static/index.html"));
    let app = Router::new()
        .route("/api/health", get(health))
        .nest(
            "/api",
            api::routes().layer(middleware::from_fn_with_state(
                state.clone(),
                require_api_key,
            )),
        )
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()))
        .fallback_service(static_service)
        .with_state(state);
    let addr: &str = &format!("0.0.0.0:{port}");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("Server started on port {port}");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }
}
