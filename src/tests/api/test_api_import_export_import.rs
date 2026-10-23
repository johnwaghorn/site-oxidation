use crate::tests::{
    TEST_PASSWORD, insert_team_sites, insert_test_user, login_and_get_cookie, parse_json_body,
    test_app, test_app_with_private_ips,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::{Row, SqlitePool};
use tower::ServiceExt;

async fn count_sites(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM sites")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_requires_authentication(pool: SqlitePool) {
    let app = test_app(pool);
    let file = serde_json::json!({"version": 1, "sites": []});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_adds_new_site_with_its_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({
        "version": 1,
        "exported_at": "2026-10-18T17:30:00Z",
        "exported_by": "0.9.0",
        "sites": [{
            "name": "Docs",
            "url": "https://docs.waghorn.tech",
            "expected_status": 204,
            "expected_text": "Welcome",
            "probe_interval_seconds": 300,
            "tls_allow_untrusted": true
        }]
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["sites"][0]["outcome"], "added");
    let site = sqlx::query("SELECT * FROM sites WHERE url = 'https://docs.waghorn.tech'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(site.get::<String, _>("name"), "Docs");
    assert_eq!(site.get::<i64, _>("expected_status"), 204);
    assert_eq!(
        site.get::<Option<String>, _>("expected_text").as_deref(),
        Some("Welcome")
    );
    assert_eq!(site.get::<i64, _>("probe_interval_seconds"), 300);
    assert!(site.get::<bool, _>("tls_allow_untrusted"));
    assert_eq!(site.get::<Option<i64>, _>("team_id"), Some(1));
    assert_eq!(site.get::<String, _>("status"), "pending", "not probed yet");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_skips_site_already_in_team(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Blog", "url": "https://blog.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["sites"][0]["outcome"], "skipped");
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_skips_url_repeated_in_file(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"},
        {"name": "Docs again", "url": "https://docs.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["sites"][0]["outcome"], "added");
    assert_eq!(body["sites"][1]["outcome"], "skipped");
    assert_eq!(count_sites(&pool).await, 4);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_into_team_user_is_not_in_is_forbidden(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=2")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_user_import_requires_a_team(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_rejects_unsupported_version(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 2, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["message"],
        "Unsupported export version 2, this instance reads version 1"
    );
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_with_one_invalid_site_imports_nothing(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"},
        {"name": "Broken", "url": "ftp://files.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("sites[1].url: site url must start with https:// or http://"),
        "{body}"
    );
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_import_rejects_private_ip_sites(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app_with_private_ips(pool.clone(), false);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Docs", "url": "https://docs.waghorn.tech"},
        {"name": "Router", "url": "http://192.168.1.1/status"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import?team_id=1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["message"],
        "sites[1].url: Private/internal IP addresses are not allowed"
    );
    assert_eq!(count_sites(&pool).await, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_imports_without_a_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let file = serde_json::json!({"version": 1, "sites": [
        {"name": "Legacy", "url": "https://legacy.waghorn.tech"},
        {"name": "Docs", "url": "https://docs.waghorn.tech"}
    ]});
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites/import")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(file.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let outcomes: Vec<&str> = body["sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| site["outcome"].as_str().unwrap())
        .collect();
    assert_eq!(outcomes, ["skipped", "added"]);
    let team_id: Option<i64> =
        sqlx::query_scalar("SELECT team_id FROM sites WHERE url = 'https://docs.waghorn.tech'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(team_id, None);
}
