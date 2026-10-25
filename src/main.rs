use crate::auth::{Authenticator, delete_expired_sessions};
use crate::notifications::Notifier;
use crate::security::rate_limit::LoginRateLimiter;
use crate::security::resolver::SafeResolver;
use anyhow::{Context, Result};
use api::ApiDoc;
use axum::{Router, routing::any};
use config::AppConfig;
use monitoring::run_due_site_checks;
use password_auth::generate_hash;
use reqwest::Client;
use sqlx::SqlitePool;
use state::AppState;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use tower_http::services::{ServeDir, ServeFile};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod api;
mod auth;
mod canary;
mod config;
mod db;
mod models;
mod monitoring;
mod notifications;
mod probe;
mod security;
mod state;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let config = AppConfig::from_env().context("Failed to load app config from env")?;
    let pool = db::init_db(&config.database_path).await.with_context(|| {
        format!(
            "Failed to initialise db connection with {}",
            config.database_path.display()
        )
    })?;
    spawn_expired_session_cleanup(pool.clone());
    let key = security::session_key::get_or_create_key(&config.session_key_path, &config.data_dir)
        .context("Failed to load or create session key")?;
    let dummy_hash: String = task::spawn_blocking(|| generate_hash("__dummy__"))
        .await
        .context("Failed to generate dummy password hash")?;
    let authenticator = Authenticator::new(pool.clone(), dummy_hash, key, config.cookie_secure);
    let login_limiter = Arc::new(LoginRateLimiter::new(5, Duration::from_mins(1)));
    let admin_limiter = Arc::new(LoginRateLimiter::new(20, Duration::from_mins(1)));
    spawn_rate_limit_pruner(Arc::clone(&login_limiter), Arc::clone(&admin_limiter));
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
    let notifier = Notifier::new(notification_client, config.smtp_allow_private_hosts);
    let state = AppState {
        pool: pool.clone(),
        config: config.clone(),
        login_limiter,
        admin_limiter,
        canary_client: verifying_client.clone(),
        notifier: notifier.clone(),
        authenticator,
    };
    let static_service = ServeDir::new("static").fallback(ServeFile::new("static/index.html"));
    let health_routes = api::healthcheck::health_routes();
    let auth_routes = api::auth::auth_routes();
    let setup_routes = api::setup::setup_routes();
    let site_routes = api::sites::site_routes();
    let import_export_routes = api::import_export::import_export_routes();
    let team_routes = api::teams::team_routes();
    let admin_routes = api::admin::admin_routes();
    let version_routes = api::version::version_routes();
    let mut app = Router::new()
        .nest("/api", health_routes)
        .nest("/api", setup_routes)
        .nest(
            "/api",
            Router::new()
                .merge(auth_routes)
                .merge(site_routes)
                .merge(import_export_routes)
                .merge(team_routes)
                .merge(admin_routes)
                .merge(version_routes),
        )
        .route("/api/{*path}", any(api::errors::api_not_found))
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
    spawn_due_site_checker(
        verifying_client,
        untrusted_client,
        pool.clone(),
        config,
        notifier.clone(),
    );
    spawn_notification_outbox_worker(notifier, pool);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .context("Axum server terminated with an error")?;
    Ok(())
}

fn spawn_expired_session_cleanup(pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_mins(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            if let Err(err) = delete_expired_sessions(&pool).await {
                tracing::error!("Failed to delete expired sessions: {err}");
            }
        }
    });
}

fn spawn_rate_limit_pruner(
    login_limiter: Arc<LoginRateLimiter>,
    admin_limiter: Arc<LoginRateLimiter>,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_mins(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            login_limiter.prune_expired();
            admin_limiter.prune_expired();
        }
    });
}

fn spawn_due_site_checker(
    verifying_client: Client,
    untrusted_client: Client,
    pool: SqlitePool,
    config: AppConfig,
    notifier: Notifier,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            run_due_site_checks(
                &verifying_client,
                &untrusted_client,
                &pool,
                &config,
                &notifier,
            )
            .await;
        }
    });
}

// Notification delivery is independent of probe transitions so queued
// alerts survive transient failures and process restarts.
fn spawn_notification_outbox_worker(notifier: Notifier, pool: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            notifier.process_outbox(&pool).await;
        }
    });
}

#[cfg(test)]
mod tests;
