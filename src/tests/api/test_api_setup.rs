use crate::tests::{
    LOOPBACK_IP, PUBLIC_IP, TEST_PASSWORD, capture_warn_logs, insert_test_user, parse_json_body,
    test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

fn bootstrap_request(ip: [u8; 4]) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/setup/bootstrap")
        .body(Body::empty())
        .unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            ip, 0,
        ))));
    request
}

fn status_request() -> Request<Body> {
    Request::builder()
        .uri("/setup/status")
        .body(Body::empty())
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_setup_status_returns_true_when_no_users(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app.oneshot(status_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["setup_required"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bootstrap_and_setup_status(pool: SqlitePool) {
    let app = test_app(pool.clone());
    let response = app
        .clone()
        .oneshot(bootstrap_request(LOOPBACK_IP))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json_body(response).await;
    assert_eq!(body["username"], "admin");
    assert!(body["password"].as_str().unwrap().len() >= 64);
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    let (role, must_change_password): (String, bool) =
        sqlx::query_as("SELECT role, must_change_password FROM users WHERE username = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(role, "admin");
    assert!(must_change_password);
    let response = app.oneshot(status_request()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["setup_required"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bootstrap_rejected_from_public_ip(pool: SqlitePool) {
    let app = test_app(pool);
    let (logs, _guard) = capture_warn_logs();
    let response = app.oneshot(bootstrap_request(PUBLIC_IP)).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = parse_json_body(response).await;
    assert_eq!(
        body.get("error").and_then(serde_json::Value::as_str),
        Some("forbidden")
    );
    let message = body
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(message.contains("BOOTSTRAP_TRUSTED_IPS"));
    assert!(message.contains("BOOTSTRAP_REQUIRE_PRIVATE_IP=false"));
    let output = logs.output();
    assert!(output.contains("Rejected bootstrap attempt from an untrusted IP"));
    assert!(output.contains("client_ip=8.8.8.8"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_bootstrap_returns_409_when_users_exist(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let response = app.oneshot(bootstrap_request(LOOPBACK_IP)).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_concurrent_bootstrap_yields_exactly_one_admin(pool: SqlitePool) {
    let app = test_app(pool.clone());
    let futures: Vec<_> = (0..10)
        .map(|_| {
            let app = app.clone();
            async move {
                app.oneshot(bootstrap_request(LOOPBACK_IP))
                    .await
                    .unwrap()
                    .status()
            }
        })
        .collect();
    let statuses = futures::future::join_all(futures).await;
    let created = statuses
        .iter()
        .filter(|s| **s == StatusCode::CREATED)
        .count();
    let conflicts = statuses
        .iter()
        .filter(|s| **s == StatusCode::CONFLICT)
        .count();
    assert_eq!(created, 1);
    assert_eq!(conflicts, 9);
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
