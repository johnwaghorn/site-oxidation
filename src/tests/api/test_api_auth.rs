use crate::tests::{
    TEST_NEW_PASSWORD, TEST_PASSWORD, WRONG_PASSWORD, build_change_password_request,
    build_login_request, extract_cookies, insert_test_user, login_and_get_cookie, parse_json_body,
    test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tracing_test::traced_test;

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
async fn test_me_includes_default_theme_preference(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
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
    assert_eq!(body["theme_preference"], "system");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_theme_preference_persists_to_me(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/auth/theme")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"theme_preference":"dark"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["theme_preference"], "dark");

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
    assert_eq!(body["theme_preference"], "dark");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_login_cookie_attributes(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let response = app
        .oneshot(build_login_request("admin", TEST_PASSWORD))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(set_cookie.starts_with("id="), "got: {set_cookie}");
    assert!(set_cookie.contains("HttpOnly"), "got: {set_cookie}");
    assert!(set_cookie.contains("SameSite=Lax"), "got: {set_cookie}");
    assert!(set_cookie.contains("Path=/"), "got: {set_cookie}");
    assert!(set_cookie.contains("Max-Age=604800"), "got: {set_cookie}");
    assert!(
        !set_cookie.contains("Secure"),
        "the test app runs without COOKIE_SECURE, got: {set_cookie}"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_tampered_cookie_is_rejected(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let mut tampered = cookie.clone();
    let last = tampered.pop().unwrap();
    tampered.push(if last == 'a' { 'b' } else { 'a' });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("cookie", &tampered)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "a cookie with a broken signature must not authenticate"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_logout_ends_session(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/logout")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let removal = response
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    assert!(
        removal.starts_with("id=;") && removal.contains("Max-Age=0"),
        "logout must clear the session cookie, got: {removal}"
    );
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
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "the old cookie must not work after logout"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_password_change_clears_must_change_and_issues_new_session(pool: SqlitePool) {
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
    let cookie = extract_cookies(&response);
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
    assert_eq!(response.status(), StatusCode::OK);
    login_and_get_cookie(&app, "admin", TEST_NEW_PASSWORD).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_password_change_logs_out_other_sessions(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let other_cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
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
    let new_cookie = extract_cookies(&response);
    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(session_count, 1, "old sessions are replaced by one new one");
    for old_cookie in [&cookie, &other_cookie] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/me")
                    .header("cookie", old_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "both the original and other-device sessions must be rejected"
        );
    }
    let response = app
        .oneshot(
            Request::builder()
                .uri("/auth/me")
                .header("cookie", &new_cookie)
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
#[traced_test]
async fn test_login_with_invalid_credentials_fails(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let response = app
        .oneshot(build_login_request("admin", WRONG_PASSWORD))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(logs_contain(
        "Failed login attempt for 'admin' from 127.0.0.1"
    ));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_login_rate_limit_is_logged(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    for _ in 0..5 {
        let response = app
            .clone()
            .oneshot(build_login_request("admin", WRONG_PASSWORD))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    assert!(logs_contain("Login rate limit reached for 127.0.0.1"));
}
