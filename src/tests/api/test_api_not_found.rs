use crate::tests::{parse_json_body, test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_unknown_api_path_returns_json_not_found(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = parse_json_body(response).await;
    assert_eq!(body["error"], "not_found");
    assert_eq!(body["message"], "Endpoint not found");
}
