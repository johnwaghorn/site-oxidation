use crate::tests::{
    TEST_PASSWORD, TEST_SITE_NAME, TEST_SITE_URL, insert_test_user, login_and_get_cookie,
    parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_create_and_list_teams(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Team Alpha"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json_body(response).await;
    assert_eq!(body["name"], "Team Alpha");
    assert_eq!(body["member_count"], 0);
    assert_eq!(body["site_count"], 0);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/teams")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_duplicate_team_returns_409(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Team Alpha"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_rename_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Old Name')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/admin/teams/1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"New Name"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_rename_nonexistent_team_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/admin/teams/9999")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Whatever"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_empty_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/teams/1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cannot_delete_team_with_sites(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'pending', 1)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/teams/1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_add_and_remove_team_member(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams/1/members")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"user_id":{user_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/teams/1/members/{user_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_add_duplicate_member_returns_409(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams/1/members")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"user_id":{user_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_add_member_to_nonexistent_team_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams/9999/members")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"user_id":{user_id}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_add_nonexistent_user_to_team_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/teams/1/members")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"user_id":9999}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_remove_nonexistent_membership_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team Alpha')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/teams/1/members/9999")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
