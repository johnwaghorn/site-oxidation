use crate::tests::{
    TEST_PASSWORD, insert_test_user, login_and_get_cookie, parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn insert_team_sites(pool: &SqlitePool, user_id: i64) {
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A'), ('Team B')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(concat!(
        "INSERT INTO sites (name, url, expected_status, expected_text, probe_interval_seconds, ",
        "tls_allow_untrusted, status, team_id) VALUES ",
        "('Blog', 'https://blog.waghorn.tech', 204, 'Welcome', 120, 1, 'down', 1), ",
        "('Shop', 'https://shop.waghorn.tech', 200, NULL, 60, 0, 'up', 2), ",
        "('Legacy', 'https://legacy.waghorn.tech', 200, NULL, 60, 0, 'pending', NULL)"
    ))
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn test_export_requires_authentication(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sites/export")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_exports_only_their_team_sites_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sites/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["version"], 1);
    assert_eq!(body["exported_by"], env!("CARGO_PKG_VERSION"));
    assert!(body["exported_at"].is_string());
    assert_eq!(
        body["sites"],
        serde_json::json!([{
            "name": "Blog",
            "url": "https://blog.waghorn.tech",
            "expected_status": 204,
            "expected_text": "Welcome",
            "probe_interval_seconds": 120,
            "tls_allow_untrusted": true,
        }])
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_exports_all_sites(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/sites/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let names: Vec<&str> = body["sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Blog", "Shop", "Legacy"]);
}
