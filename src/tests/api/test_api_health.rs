use crate::tests::{parse_json_body, test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tracing_test::traced_test;

#[sqlx::test(migrations = "./migrations")]
async fn test_health_returns_ok_when_database_reachable(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["checks"]["database"]["status"], "healthy");
    assert!(body.get("version").is_none());
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_health_hides_database_error_details(pool: SqlitePool) {
    pool.close().await;
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = parse_json_body(response).await;
    assert_eq!(body["status"], "unhealthy");
    assert_eq!(body["checks"]["database"]["error"], "Database unavailable");
    assert!(logs_contain("Healthcheck database query failed"));
}
