use crate::models::site::{SiteRow, SiteStatus};
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

pub async fn insert_test_site(pool: &SqlitePool, status: SiteStatus) -> SiteRow {
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
