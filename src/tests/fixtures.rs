use crate::canary;
use crate::models::notifications::TeamNotificationConfig;
use crate::models::site::{SiteRow, SiteStatus};
use axum::Router;
use password_auth::generate_hash;
use sqlx::SqlitePool;

pub const TEST_SITE_NAME: &str = "Waghorn Technology Ltd";
pub const TEST_SITE_URL: &str = "https://waghorn.tech";
pub const TEST_SITE_EXPECTED_STATUS: i64 = 200;
pub const TEST_PROBE_INTERVAL_SECONDS: i64 = 60;
pub const TEST_PASSWORD: &str = "super-secret-password-123";
pub const TEST_NEW_PASSWORD: &str = "new-super-secret-password-123";
pub const WRONG_PASSWORD: &str = "wrong-password";
pub const LOOPBACK_IP: [u8; 4] = [127, 0, 0, 1];
pub const PUBLIC_IP: [u8; 4] = [8, 8, 8, 8];

pub async fn insert_test_user(
    pool: &SqlitePool,
    username: &str,
    password: &str,
    role: &str,
    must_change_password: bool,
) -> i64 {
    let hash = generate_hash(password);
    sqlx::query_scalar(
        "INSERT INTO users (username, password, role, must_change_password) \
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(username)
    .bind(&hash)
    .bind(role)
    .bind(must_change_password)
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn authenticated_admin_app(
    pool: SqlitePool,
    allow_private_ips: bool,
) -> (Router, String) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = super::app_builder::test_app_with_private_ips(pool, allow_private_ips);
    let cookie = super::auth_helpers::login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    (app, cookie)
}

pub async fn insert_test_site(pool: &SqlitePool, status: SiteStatus) -> SiteRow {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO sites (name, url, expected_status, status) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .bind(TEST_SITE_EXPECTED_STATUS)
    .bind(status)
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
        tls_allow_untrusted: false,
        cert_status: None,
        notifications: TeamNotificationConfig::default(),
    }
}

pub async fn insert_team_sites(pool: &SqlitePool, user_id: i64) {
    let team_a = insert_team(pool, "Team A").await;
    let team_b = insert_team(pool, "Team B").await;
    add_team_member(pool, team_a, user_id).await;
    sqlx::query(concat!(
        "INSERT INTO sites (name, url, expected_status, expected_text, probe_interval_seconds, ",
        "tls_allow_untrusted, status, team_id) VALUES ",
        "('Blog', 'https://blog.waghorn.tech', 204, 'Welcome', 120, 1, 'down', ?1), ",
        "('Shop', 'https://shop.waghorn.tech', 200, NULL, 60, 0, 'up', ?2), ",
        "('Legacy', 'https://legacy.waghorn.tech', 200, NULL, 60, 0, 'pending', NULL)"
    ))
    .bind(team_a)
    .bind(team_b)
    .execute(pool)
    .await
    .unwrap();
}

pub async fn insert_team(pool: &SqlitePool, name: &str) -> i64 {
    sqlx::query_scalar("INSERT INTO teams (name) VALUES (?) RETURNING id")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

pub async fn add_team_member(pool: &SqlitePool, team_id: i64, user_id: i64) {
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
}

pub async fn count_sites(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM sites")
        .fetch_one(pool)
        .await
        .unwrap()
}

pub async fn count_outages(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM outages")
        .fetch_one(pool)
        .await
        .unwrap()
}

pub async fn count_sessions(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await
        .unwrap()
}

pub async fn enable_canary(pool: &SqlitePool, url: &str) {
    canary::update_settings(
        pool,
        &canary::CanaryConfig {
            enabled: true,
            url: Some(url.to_owned()),
            timeout_secs: 1,
        },
    )
    .await
    .unwrap();
}
