use crate::tests::{
    TEST_PASSWORD, TEST_PROBE_INTERVAL_SECONDS, TEST_SITE_NAME, TEST_SITE_URL, insert_test_user,
    login_and_get_cookie, parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_requires_team_id_on_create(pool: SqlitePool) {
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
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
    );
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_can_create_with_team_id(pool: SqlitePool) {
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
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":1}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_cannot_access_other_teams_sites(pool: SqlitePool) {
    let user1_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_test_user(&pool, "user2", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO teams (name) VALUES ('Team B')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user1_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 2)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_access_any_teams_sites(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 1)",
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
                .uri("/sites/1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_cannot_create_site_for_other_team(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO teams (name) VALUES ('Team B')")
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
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":2}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_different_teams_can_monitor_the_same_url(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO teams (name) VALUES ('Team B')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 2)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":1}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_same_team_cannot_duplicate_a_url(pool: SqlitePool) {
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
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 1)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let payload = format!(
        r#"{{"name":"A Different Name","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":1}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_update_site_to_url_used_in_same_team_returns_409(pool: SqlitePool) {
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
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES ('Site One', 'https://one.waghorn.tech', 200, 'up', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let target_id: i64 = sqlx::query_scalar(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES ('Site Two', 'https://two.waghorn.tech', 200, 'up', 1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let payload = format!(
        r#"{{"name":"Site Two","url":"https://one.waghorn.tech","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":1}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/sites/{target_id}"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_list_shows_only_own_teams_sites(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO teams (name) VALUES ('Team B')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (1, ?)")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES ('Site A', 'https://a.com', 200, 'up', 1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES ('Site B', 'https://b.com', 200, 'up', 2)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
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
    let body = parse_json_body(response).await;
    let sites = body["data"].as_array().unwrap();
    assert_eq!(sites.len(), 1);
    assert_eq!(sites[0]["name"], "Site A");
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 20);
    assert_eq!(body["total"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_list_includes_team_name(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO teams (name) VALUES ('Team A')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 1)",
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
                .uri("/sites")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["data"][0]["team_name"], "Team A");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_list_includes_team_name(pool: SqlitePool) {
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
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'up', 1)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
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
    let body = parse_json_body(response).await;
    assert_eq!(body["data"][0]["team_name"], "Team A");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_unassigned_site_has_null_team_name(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO sites (name, url, expected_status, status) VALUES (?, ?, 200, 'up')")
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
                .uri("/sites")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert!(body["data"][0]["team_name"].is_null());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site_response_includes_team_name(pool: SqlitePool) {
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
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS},"team_id":1}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = parse_json_body(response).await;
    assert_eq!(body["team_name"], "Team A");
}
