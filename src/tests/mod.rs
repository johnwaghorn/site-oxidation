mod api;

use crate::api::auth::require_api_key;
use crate::config::AppConfig;
use crate::models::{SiteRow, SiteStatus};
use crate::state::AppState;
use axum::Router;
use axum::middleware;
use sqlx::SqlitePool;

pub const TEST_API_KEY: &str = "test-secret";
pub const TEST_SITE_NAME: &str = "Waghorn Technology Ltd";
pub const TEST_SITE_URL: &str = "https://waghorn.tech";
pub const TEST_PROBE_INTERVAL_SECONDS: i64 = 60;

pub fn test_config() -> AppConfig {
    AppConfig {
        api_key: TEST_API_KEY.to_string(),
        database_path: ":memory:".to_string(),
        server_port: 8080,
        probe_timeout_secs: 30,
        probe_retry_count: 2,
        probe_retry_delay_ms: 3000,
        allow_private_ips: true,
    }
}

pub fn test_app(pool: SqlitePool) -> Router {
    let state = AppState {
        pool,
        config: test_config(),
    };
    crate::api::routes()
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_key,
        ))
        .with_state(state)
}

pub async fn insert_test_site(pool: &SqlitePool, status: SiteStatus) -> SiteRow {
    pub const TEST_SITE_EXPECTED_STATUS: i64 = 200;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO sites (name, url, expected_status, status) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .bind(TEST_SITE_EXPECTED_STATUS)
    .bind(&status)
    .fetch_one(pool)
    .await
    .unwrap();
    SiteRow {
        id,
        name: TEST_SITE_NAME.to_string(),
        url: TEST_SITE_URL.to_string(),
        expected_status: TEST_SITE_EXPECTED_STATUS,
        expected_text: None,
        status,
        probe_interval_seconds: TEST_PROBE_INTERVAL_SECONDS,
    }
}
