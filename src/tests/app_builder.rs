use crate::auth::Authenticator;
use crate::config::AppConfig;
use crate::state::AppState;
use axum::Router;
use axum_extra::extract::cookie::Key;
use password_auth::generate_hash;
use sqlx::SqlitePool;

pub fn test_config(probe_allow_private_ips: bool) -> AppConfig {
    AppConfig {
        allowed_origin: None,
        bootstrap_require_private_ip: true,
        bootstrap_trusted_ips: Vec::new(),
        cert_critical_days: 7,
        cert_warn_days: 30,
        cookie_secure: false,
        data_dir: std::path::PathBuf::from(":memory:"),
        database_path: std::path::PathBuf::from(":memory:"),
        enable_swagger_ui: false,
        probe_allow_private_ips,
        probe_body_size_limit_bytes: 1_048_576,
        probe_max_concurrent_checks: 20,
        probe_retry_count: 2,
        probe_retry_delay_ms: 3000,
        probe_timeout_secs: 30,
        probe_user_agent: "SiteOxidation/test".to_owned(),
        server_port: 8080,
        smtp_allow_private_hosts: probe_allow_private_ips,
        session_key_path: std::path::PathBuf::from(":memory:/session.key"),
    }
}

pub fn test_app_with_private_ips(pool: SqlitePool, allow_private_ips: bool) -> Router {
    let dummy_hash = generate_hash("__dummy__");
    let authenticator = Authenticator::new(pool.clone(), dummy_hash, Key::generate(), false);
    let config = test_config(allow_private_ips);
    let notifier = crate::notifications::Notifier::new(
        reqwest::Client::new(),
        config.smtp_allow_private_hosts,
    );
    let state = AppState {
        pool,
        config,
        login_limiter: std::sync::Arc::new(crate::security::rate_limit::LoginRateLimiter::new(
            5,
            std::time::Duration::from_mins(1),
        )),
        admin_limiter: std::sync::Arc::new(crate::security::rate_limit::LoginRateLimiter::new(
            10,
            std::time::Duration::from_mins(1),
        )),
        canary_client: reqwest::Client::new(),
        notifier,
        authenticator,
    };
    let auth_routes = crate::api::auth::auth_routes();
    let site_routes = crate::api::sites::site_routes();
    let team_routes = crate::api::teams::team_routes();
    let admin_routes = crate::api::admin::admin_routes();
    let setup_routes = crate::api::setup::setup_routes();
    let health_routes = crate::api::healthcheck::health_routes();
    Router::new()
        .merge(health_routes)
        .merge(setup_routes)
        .merge(
            Router::new()
                .merge(auth_routes)
                .merge(site_routes)
                .merge(team_routes)
                .merge(admin_routes),
        )
        .with_state(state)
}

pub fn test_app(pool: SqlitePool) -> Router {
    test_app_with_private_ips(pool, true)
}

pub fn test_app_with_cors(pool: SqlitePool, allowed_origin: &str) -> Router {
    let mut config = test_config(true);
    config.allowed_origin = Some(allowed_origin.to_string());
    let cors = crate::security::cors::cors_layer(&config).expect("Invalid test ALLOWED_ORIGIN");

    let dummy_hash = generate_hash("__dummy__");
    let authenticator = Authenticator::new(pool.clone(), dummy_hash, Key::generate(), false);
    let notifier = crate::notifications::Notifier::new(
        reqwest::Client::new(),
        config.smtp_allow_private_hosts,
    );
    let state = AppState {
        pool,
        config,
        login_limiter: std::sync::Arc::new(crate::security::rate_limit::LoginRateLimiter::new(
            5,
            std::time::Duration::from_mins(1),
        )),
        admin_limiter: std::sync::Arc::new(crate::security::rate_limit::LoginRateLimiter::new(
            10,
            std::time::Duration::from_mins(1),
        )),
        canary_client: reqwest::Client::new(),
        notifier,
        authenticator,
    };
    let auth_routes = crate::api::auth::auth_routes();
    let site_routes = crate::api::sites::site_routes();
    let team_routes = crate::api::teams::team_routes();
    let admin_routes = crate::api::admin::admin_routes();
    let setup_routes = crate::api::setup::setup_routes();
    let health_routes = crate::api::healthcheck::health_routes();
    Router::new()
        .merge(health_routes)
        .merge(setup_routes)
        .merge(
            Router::new()
                .merge(auth_routes)
                .merge(site_routes)
                .merge(team_routes)
                .merge(admin_routes),
        )
        .layer(cors)
        .with_state(state)
}
