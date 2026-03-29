use crate::config::AppConfig;
use crate::security::rate_limit::LoginRateLimiter;
use axum::extract::FromRef;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: AppConfig,
    pub login_limiter: Arc<LoginRateLimiter>,
    pub admin_limiter: Arc<LoginRateLimiter>,
}

pub struct AdminLimiter(pub Arc<LoginRateLimiter>);

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

impl FromRef<AppState> for Arc<LoginRateLimiter> {
    fn from_ref(state: &AppState) -> Self {
        Arc::clone(&state.login_limiter)
    }
}

impl FromRef<AppState> for AdminLimiter {
    fn from_ref(state: &AppState) -> Self {
        AdminLimiter(Arc::clone(&state.admin_limiter))
    }
}
