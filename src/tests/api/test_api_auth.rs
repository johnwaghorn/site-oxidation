use crate::tests::{
    TEST_NEW_PASSWORD, TEST_PASSWORD, WRONG_PASSWORD, build_change_password_request,
    build_login_request, insert_test_user, login_and_get_cookie, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_password_change_clears_must_change_and_keeps_session(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", true).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/sites")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .clone()
        .oneshot(build_change_password_request(
            &cookie,
            TEST_PASSWORD,
            TEST_NEW_PASSWORD,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_password_change_with_wrong_current_password_fails(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(build_change_password_request(
            &cookie,
            WRONG_PASSWORD,
            TEST_NEW_PASSWORD,
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_with_invalid_credentials_fails(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let response = app
        .oneshot(build_login_request("admin", WRONG_PASSWORD))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
