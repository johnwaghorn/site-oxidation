use crate::models::site::SiteStatus;
use crate::tests::{
    TEST_PASSWORD, TEST_SITE_NAME, TEST_SITE_URL, authenticated_admin_app, insert_test_site,
    insert_test_user, login_and_get_cookie, parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_outages_returns_inserted_outage(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query("INSERT INTO outages (site_id, http_status) VALUES (?, ?)")
        .bind(1)
        .bind(500)
        .execute(&pool)
        .await
        .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1/outages")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 20);
    assert_eq!(body["total"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_outages_returns_empty_for_site_with_no_outages(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1/outages")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_outages_returns_404_for_nonexistent_site(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/9999/outages")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_cannot_see_outages_for_other_teams_sites(pool: SqlitePool) {
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
        "INSERT INTO sites (name, url, expected_status, status, team_id) VALUES (?, ?, 200, 'down', 2)",
    )
    .bind(TEST_SITE_NAME)
    .bind(TEST_SITE_URL)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO outages (site_id, http_status) VALUES (1, 500)")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "user1", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1/outages")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
