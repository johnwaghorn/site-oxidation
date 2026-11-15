use crate::models::site::SiteStatus;
use crate::tests::{
    TEST_PASSWORD, TEST_SITE_NAME, TEST_SITE_URL, add_team_member, authenticated_admin_app,
    insert_team, insert_test_site, insert_test_user, login_and_get_cookie, parse_json_body,
    test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_outages_returns_inserted_outage(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query("INSERT INTO outages (site_id, http_status, expected_status) VALUES (?, ?, ?)")
        .bind(1)
        .bind(500)
        .bind(200)
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
    assert_eq!(body["data"][0]["http_status"], 500);
    assert_eq!(body["data"][0]["expected_status"], 200);
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
    insert_team(&pool, "Team A").await;
    insert_team(&pool, "Team B").await;
    add_team_member(&pool, 1, user_id).await;
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

#[sqlx::test(migrations = "./migrations")]
async fn test_outage_export_returns_every_outage_oldest_first_as_csv(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query(
        "INSERT INTO outages (site_id, http_status, expected_status, error_message, started_at, ended_at) VALUES \
         (1, NULL, 200, 'connection refused', '2026-11-12 08:00:00', NULL), \
         (1, 503, 200, 'Expected HTTP status 200, received 503', '2026-11-10 09:30:00', '2026-11-10 09:42:30')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1/outages/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "text/csv; charset=utf-8"
    );
    assert_eq!(
        response.headers()["content-disposition"],
        format!(
            "attachment; filename=\"site-oxidation-outages-Waghorn-Technology-Ltd-{}.csv\"",
            Utc::now().date_naive()
        )
        .as_str()
    );
    let csv = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(
        String::from_utf8(csv.to_vec()).unwrap(),
        "started_at,ended_at,duration_seconds,http_status,expected_status,error_message\n\
         2026-11-10T09:30:00Z,2026-11-10T09:42:30Z,750,503,200,\"Expected HTTP status 200, received 503\"\n\
         2026-11-12T08:00:00Z,,,,200,connection refused\n"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outage_export_includes_outages_beyond_the_first_chunk_read(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    sqlx::query(
        "WITH RECURSIVE numbers(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM numbers WHERE n < 2500) \
         INSERT INTO outages (site_id, http_status, expected_status, started_at, ended_at) \
         SELECT 1, 500, 200, datetime('2026-01-01', '+' || n || ' hours'), \
                datetime('2026-01-01', '+' || n || ' hours', '+5 minutes') FROM numbers",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites/1/outages/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let csv = response.into_body().collect().await.unwrap().to_bytes();
    let csv = String::from_utf8(csv.to_vec()).unwrap();
    let rows: Vec<&str> = csv.lines().collect();
    assert_eq!(rows.len(), 2501);
    assert_eq!(
        rows[1],
        "2026-01-01T01:00:00Z,2026-01-01T01:05:00Z,300,500,200,"
    );
    assert_eq!(
        rows[2500],
        "2026-04-15T04:00:00Z,2026-04-15T04:05:00Z,300,500,200,"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_admin_cannot_export_outages_for_other_teams_sites(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "user1", TEST_PASSWORD, "user", false).await;
    insert_team(&pool, "Team A").await;
    insert_team(&pool, "Team B").await;
    add_team_member(&pool, 1, user_id).await;
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
                .uri("/sites/1/outages/export")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
