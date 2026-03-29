use crate::models::site::SiteStatus;
use crate::tests::{
    TEST_PASSWORD, TEST_PROBE_INTERVAL_SECONDS, TEST_SITE_NAME, TEST_SITE_URL, insert_test_site,
    insert_test_user, login_and_get_cookie, parse_json_body, test_app, test_app_with_private_ips,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn authenticated_admin_app(pool: SqlitePool) -> (axum::Router, String) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    (app, cookie)
}

async fn authenticated_admin_app_with_private_ips(
    pool: SqlitePool,
    allow_private_ips: bool,
) -> (axum::Router, String) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app_with_private_ips(pool, allow_private_ips);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    (app, cookie)
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_returns_inserted_site(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool).await;
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains(TEST_SITE_NAME));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
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
async fn test_create_site_invalid_payload_returns_422(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"","url":"not a url"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site_private_ip_rejected_when_disabled(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app_with_private_ips(pool, false).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"http://127.0.0.1:8080","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
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
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site_private_ip_allowed_when_enabled(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app_with_private_ips(pool, true).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"http://127.0.0.1:8080","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
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
async fn test_update_site_private_ip_rejected_when_disabled(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app_with_private_ips(pool, false).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"http://127.0.0.1:8080","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/sites/1")
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
async fn test_update_site_private_ip_allowed_when_enabled(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app_with_private_ips(pool, true).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"http://127.0.0.1:8080","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/sites/1")
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_site_cascades_outages(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    sqlx::query("INSERT INTO outages (site_id, http_status) VALUES (?, ?)")
        .bind(1)
        .bind(500)
        .execute(&pool)
        .await
        .unwrap();
    let (app, cookie) = authenticated_admin_app(pool.clone()).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/sites/1")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

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
async fn test_non_admin_cannot_create_for_other_team(pool: SqlitePool) {
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
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_create_site_without_team_id(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool).await;
    let payload = format!(
        r#"{{"name":"{TEST_SITE_NAME}","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
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
