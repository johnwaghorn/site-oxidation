use crate::tests::{
    TEST_PASSWORD, insert_test_user, login_and_get_cookie, parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_version_requires_authentication(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/version")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_version_returns_crate_version(pool: SqlitePool) {
    insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/version")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
}
