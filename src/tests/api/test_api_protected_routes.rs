use crate::tests::{TEST_PASSWORD, insert_test_user, login_and_get_cookie, test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

const URI: &str = "/sites";

#[sqlx::test(migrations = "./migrations")]
async fn test_unauthenticated_request_returns_401(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(URI)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_authenticated_session_can_access_protected_routes(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(URI)
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_deactivated_user_loses_session_access(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    sqlx::query("UPDATE users SET active = 0 WHERE username = 'admin'")
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(URI)
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_must_change_password_returns_403_on_protected_routes(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", true).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(URI)
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
