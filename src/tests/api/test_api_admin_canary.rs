use crate::canary::{self, CanaryConfig};
use crate::tests::{
    TEST_PASSWORD, TestHttpServer, insert_test_user, login_and_get_cookie, parse_json_body,
    test_app,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::SqlitePool;
use std::time::Duration;
use tower::ServiceExt;

fn settings_request(method: &str, cookie: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri("/admin/canary")
        .header("cookie", cookie)
        .header("content-type", "application/json")
        .body(body)
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_update_canary_settings(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .clone()
        .oneshot(settings_request("GET", &cookie, Body::empty()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["enabled"], false);
    assert!(body["url"].is_null());
    assert_eq!(body["timeout_secs"], 3);
    assert_eq!(body["state"], "disabled");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/canary/test")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let response = app
        .oneshot(settings_request(
            "PUT",
            &cookie,
            Body::from(
                r#"{"enabled":true,"url":"https://status.waghorn.tech/health","timeout_secs":9}"#,
            ),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["enabled"], true);
    assert_eq!(body["url"], "https://status.waghorn.tech/health");
    assert_eq!(body["state"], "unknown");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_canary_settings_reject_invalid_values(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(settings_request(
            "PUT",
            &cookie,
            Body::from(r#"{"enabled":true,"url":null,"timeout_secs":3}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let response = app
        .oneshot(settings_request(
            "PUT",
            &cookie,
            Body::from(r#"{"enabled":true,"url":"file:///etc/passwd","timeout_secs":0}"#),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_test_canary_and_record_health(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let server = TestHttpServer::start().await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(settings_request(
            "PUT",
            &cookie,
            Body::from(format!(
                r#"{{"enabled":true,"url":"{}","timeout_secs":1}}"#,
                server.base_url()
            )),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/canary/test")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["state"], "healthy");
    assert!(body["last_checked_at"].is_string());
    assert!(body["last_succeeded_at"].is_string());
    assert!(body["last_error"].is_null());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_manual_test_conflicts_when_settings_change_while_it_runs(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let hanging_server = TestHttpServer::start_hanging().await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(settings_request(
            "PUT",
            &cookie,
            Body::from(format!(
                r#"{{"enabled":true,"url":"{}","timeout_secs":1}}"#,
                hanging_server.base_url()
            )),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let updater = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        canary::update_settings(
            &pool,
            &CanaryConfig {
                enabled: true,
                url: Some("https://replacement.waghorn.tech/health".to_owned()),
                timeout_secs: 1,
            },
        )
        .await
        .unwrap();
    });
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/canary/test")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    updater.await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let body = parse_json_body(response).await;
    assert_eq!(body["error"], "conflict");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_canary_settings_require_admin(pool: SqlitePool) {
    insert_test_user(&pool, "user", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user", TEST_PASSWORD).await;
    let response = app
        .oneshot(settings_request("GET", &cookie, Body::empty()))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
