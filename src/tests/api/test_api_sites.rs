use crate::models::SiteStatus;
use crate::tests::api::test_auth_header;
use crate::tests::{TEST_SITE_NAME, TEST_SITE_URL, insert_test_site, test_app};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;
#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_returns_inserted_site(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let app = test_app(pool);
    let (auth_header_name, auth_header_value) = test_auth_header();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites")
                .header(auth_header_name, auth_header_value)
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
    let app = test_app(pool);
    let (auth_header_name, auth_header_value) = test_auth_header();
    let payload = format!(
        r#"{{"name":"{}","url":"{}"}}"#,
        TEST_SITE_NAME, TEST_SITE_URL
    );
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header(auth_header_name, auth_header_value)
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
    let app = test_app(pool);
    let (auth_header_name, auth_header_value) = test_auth_header();
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sites")
                .header(auth_header_name, auth_header_value)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"","url":"not a url"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_delete_site_cascades_outages(pool: SqlitePool) {
    let app = test_app(pool.clone());
    let (auth_header_name, auth_header_value) = test_auth_header();
    insert_test_site(&pool, SiteStatus::Up).await;
    sqlx::query("INSERT INTO outages (site_id, http_status) VALUES (?, ?)")
        .bind(1)
        .bind(500)
        .execute(&pool)
        .await
        .unwrap();
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/sites/1")
                .header(auth_header_name, auth_header_value)
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
