mod api;
mod auth_backend;
mod config;
mod db;
mod jobs;
mod models;
mod notifications;
mod probe;
mod security;
mod state;
#[cfg(test)]
mod tests;

use crate::auth_backend::Backend;
use crate::security::resolver::SafeResolver;
use anyhow::{Context, Result};
use api::ApiDoc;
use axum::Router;
use axum_login::AuthManagerLayerBuilder;
use config::AppConfig;
use jobs::check_all_sites;
use password_auth::generate_hash;
use reqwest::Client;
use state::AppState;
use std::sync::Arc;
use std::time::Duration;
use time::Duration as TimeDuration;
use tokio::task;
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::cookie::SameSite;
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = AppConfig::from_env().context("Failed to load app config from env")?;
    let pool = db::init_db(&config.database_path).await.with_context(|| {
        format!(
            "Failed to initialise db connection with {}",
            config.database_path.display()
        )
    })?;
    let session_store = SqliteStore::new(pool.clone());
    session_store
        .migrate()
        .await
        .context("Session store migration failed")?;

    let _deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(Duration::from_mins(1)),
    );
    let key = security::session_key::get_or_create_key(&config.session_key_path, &config.data_dir)
        .context("Failed to load or create session key")?;
    let dummy_hash: String = task::spawn_blocking(|| generate_hash("__dummy__"))
        .await
        .context("Failed to generate dummy password hash")?;
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(config.cookie_secure)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(TimeDuration::days(7)))
        .with_signed(key);
    let backend = Backend::new(pool.clone(), dummy_hash);
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    let login_limiter = Arc::new(security::rate_limit::LoginRateLimiter::new(
        5,
        Duration::from_mins(1),
    ));
    let admin_limiter = Arc::new(security::rate_limit::LoginRateLimiter::new(
        20,
        Duration::from_mins(1),
    ));
    let pruner_limiter = Arc::clone(&login_limiter);
    let pruner_admin_limiter = Arc::clone(&admin_limiter);
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        login_limiter,
        admin_limiter,
    };
    let resolver: Arc<SafeResolver> = Arc::new(SafeResolver {
        allow_private: config.probe_allow_private_ips,
    });
    let probe_builder = || {
        Client::builder()
            .user_agent(&config.probe_user_agent)
            .redirect(reqwest::redirect::Policy::none())
            .dns_resolver(Arc::clone(&resolver))
    };
    let verifying_client = probe_builder()
        .build()
        .context("Failed to build verifying probe client")?;
    let untrusted_client = probe_builder()
        .tls_danger_accept_invalid_certs(true)
        .tls_danger_accept_invalid_hostnames(true)
        .build()
        .context("Failed to build no-verify probe client for tls_allow_untrusted sites")?;
    let notification_client = probe_builder()
        .build()
        .context("Failed to build notification client")?;
    let notifier = notifications::Notifier::new(notification_client);
    let static_service = ServeDir::new("static").fallback(ServeFile::new("static/index.html"));
    let health_routes = api::healthcheck::health_routes();
    let auth_routes = api::auth::auth_routes();
    let setup_routes = api::setup::setup_routes();
    let site_routes = api::sites::site_routes();
    let team_routes = api::teams::team_routes();
    let admin_routes = api::admin::admin_routes();
    let mut app = Router::new()
        .nest("/api", health_routes)
        .nest("/api", setup_routes)
        .nest(
            "/api",
            Router::new()
                .merge(auth_routes)
                .merge(site_routes)
                .merge(team_routes)
                .merge(admin_routes)
                .layer(auth_layer),
        )
        .layer(security::cors::cors_layer(&config)?)
        .fallback_service(static_service)
        .with_state(state);
    if config.enable_swagger_ui {
        app =
            app.merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", ApiDoc::openapi()));
    }
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind server listener on {addr}"))?;
    tracing::info!("Server started on {}", addr);
    let checker_pool = pool.clone();
    let checker_config = config.clone();
    let checker_notifier = notifier.clone();
    // Background site checker: wakes every n seconds and probes any sites
    // whose configured check interval has elapsed.
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            check_all_sites(
                &verifying_client,
                &untrusted_client,
                &checker_pool,
                &checker_config,
                &checker_notifier,
            )
            .await;
        }
    });
    // Rate-limit cleanup worker: periodically removes expired login/admin
    // limiter entries from memory.
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_mins(5));
        loop {
            interval.tick().await;
            pruner_limiter.prune_expired();
            pruner_admin_limiter.prune_expired();
        }
    });
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .context("Axum server terminated with an error")?;
    Ok(())
}
