use crate::tests::{
    TEST_NEW_PASSWORD, TEST_PASSWORD, WRONG_PASSWORD, build_change_password_request,
    build_login_request, capture_warn_logs, insert_test_user, login_and_get_cookie,
    parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_me_includes_all_teams_without_memberships(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A'), ('Team B')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let teams = body["teams"].as_array().unwrap();
    assert_eq!(teams.len(), 2);
    assert_eq!(teams[0]["name"], "Team A");
    assert_eq!(teams[1]["name"], "Team B");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_me_includes_only_memberships(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A'), ('Team B')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let teams = body["teams"].as_array().unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0]["name"], "Team A");
}

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
    let (logs, _guard) = capture_warn_logs();
    let response = app
        .oneshot(build_login_request("admin", WRONG_PASSWORD))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let output = logs.output();
    assert!(output.contains("Failed login attempt for 'admin' from 127.0.0.1"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_rate_limit_is_logged(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let (logs, _guard) = capture_warn_logs();
    for _ in 0..5 {
        let response = app
            .clone()
            .oneshot(build_login_request("admin", WRONG_PASSWORD))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    let output = logs.output();
    assert!(output.contains("Login rate limit reached for 127.0.0.1"));
}
