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
async fn test_create_and_list_users(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
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
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"newuser","password":"temp-pass-123","role":"user","team_id":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json_body(response).await;
    assert_eq!(body["username"], "newuser");
    assert_eq!(body["role"], "user");
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 20);
    assert_eq!(body["total"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_gets_403(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
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
    for uri in ["/admin/users", "/admin/teams"] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::FORBIDDEN,
            "Expected 403 on {uri}"
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn test_unauthenticated_gets_401(pool: SqlitePool) {
    let app = test_app(pool);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_duplicate_username_returns_409(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"temp-pass-123","role":"admin"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_user_requires_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"newuser","password":"temp-pass-123","role":"user"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_user_nonexistent_team_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"newuser","password":"temp-pass-123","role":"user","team_id":999}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_admin_without_team_succeeds(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"newadmin","password":"temp-pass-123","role":"admin"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_user_with_team_adds_membership(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"newuser","password":"temp-pass-123","role":"user","team_id":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let (count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM team_members tm JOIN users u ON u.id = tm.user_id \
         WHERE u.username = 'newuser' AND tm.team_id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        count, 1,
        "new non-admin user should be a member of the team"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_duplicate_user_rolls_back(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_test_user(&pool, "dupe", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    let (users_before,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (members_before,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM team_members")
        .fetch_one(&pool)
        .await
        .unwrap();
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/admin/users")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"username":"dupe","password":"temp-pass-123","role":"user","team_id":1}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    let (users_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (members_after,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM team_members")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        users_after, users_before,
        "rolled-back duplicate create must not add a user row"
    );
    assert_eq!(
        members_after, members_before,
        "rolled-back create must not add a membership row"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_user_and_nonexistent_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{user_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"admin","active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/admin/users/9999")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"user","active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_user_cascades_memberships_and_nonexistent_returns_404(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/{user_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let membership_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM team_members WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(membership_count, 0);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/{user_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cannot_delete_self(pool: SqlitePool) {
    let admin_id = insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/{admin_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_can_delete_other_admin_when_not_last(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let admin2_id = insert_test_user(&pool, "admin2", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/users/{admin2_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cannot_demote_admin_without_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let admin2_id = insert_test_user(&pool, "admin2", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{admin2_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"user","active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_can_demote_admin_with_team(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let admin2_id = insert_test_user(&pool, "admin2", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(admin2_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{admin2_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"user","active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_reset_password_sets_must_change(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/admin/users/{user_id}/reset-password"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"temp_password":"new-temp-pass-123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let (must_change,): (bool,) =
        sqlx::query_as("SELECT must_change_password FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(must_change);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_self_guards_block_deactivate_and_demote(pool: SqlitePool) {
    let admin_id = insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let deactivate_self = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{admin_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"admin","active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(deactivate_self.status(), StatusCode::CONFLICT);
    let demote_self = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{admin_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"user","active":true}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(demote_self.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_users_filters_by_team_id(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let alice_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    insert_test_user(&pool, "bob", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('engineering')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(alice_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users?team_id=1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0]["username"], "alice");
    assert_eq!(body["total"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_users_excludes_by_team_id(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let alice_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    insert_test_user(&pool, "bob", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('engineering')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(alice_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users?exclude_team_id=1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(body["total"], 2);
    let names: Vec<&str> = data
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"alice"));
    assert!(names.contains(&"admin"));
    assert!(names.contains(&"bob"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_users_filters_by_active_status(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_test_user(&pool, "active", TEST_PASSWORD, "user", false).await;
    let inactive_id = insert_test_user(&pool, "inactive", TEST_PASSWORD, "user", false).await;
    sqlx::query("UPDATE users SET active = 0 WHERE id = ?")
        .bind(inactive_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users?active=true")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let names: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"admin"));
    assert!(names.contains(&"active"));
    assert!(!names.contains(&"inactive"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_users_filters_by_search(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    insert_test_user(&pool, "alastair", TEST_PASSWORD, "user", false).await;
    insert_test_user(&pool, "bob", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users?search=al")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(body["total"], 2);
    let names: Vec<&str> = data
        .iter()
        .map(|u| u["username"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"alice"));
    assert!(names.contains(&"alastair"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_users_search_blank_is_ignored(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/users?search=%20%20")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["total"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_can_deactivate_other_admin_when_not_last(pool: SqlitePool) {
    insert_test_user(&pool, "admin1", TEST_PASSWORD, "admin", false).await;
    let admin2_id = insert_test_user(&pool, "admin2", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/admin/users/{admin2_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"role":"admin","active":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
