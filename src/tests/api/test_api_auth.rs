use crate::tests::api::test_auth_header;
use crate::tests::test_app;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_api_auth_invalid_key_returns_401(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites")
                .header("Authorization", "Bearer completely_wrong_key")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
}

#[sqlx::test(migrations = "./migrations")]
async fn test_api_auth_valid_request_returns_200(pool: SqlitePool) {
    let app = test_app(pool);
    let (header_name, header_value) = test_auth_header();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites")
                .header(header_name, header_value)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK)
}
