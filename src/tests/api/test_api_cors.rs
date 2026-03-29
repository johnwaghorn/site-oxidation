use crate::tests::test_app_with_cors;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

const ALLOWED: &str = "https://myapp.example.com";
const DISALLOWED: &str = "https://evil.example.com";

#[sqlx::test(migrations = "./migrations")]
async fn test_cors_preflight_allowed_origin(pool: SqlitePool) {
    let app = test_app_with_cors(pool, ALLOWED);
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("origin", ALLOWED)
                .header("access-control-request-method", "POST")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        ALLOWED
    );
    assert!(
        response
            .headers()
            .get("access-control-allow-credentials")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("true")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cors_preflight_disallowed_origin_not_echoed(pool: SqlitePool) {
    let app = test_app_with_cors(pool, ALLOWED);
    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .header("origin", DISALLOWED)
                .header("access-control-request-method", "POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .unwrap()
        .to_str()
        .unwrap();
    assert_ne!(allow_origin, DISALLOWED);
    assert_eq!(allow_origin, ALLOWED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cors_actual_request_includes_allow_origin(pool: SqlitePool) {
    let app = test_app_with_cors(pool, ALLOWED);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .header("origin", ALLOWED)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        ALLOWED
    );
}
