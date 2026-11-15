use crate::notifications::delivery::PendingDelivery;
use crate::tests::{
    TEST_PASSWORD, insert_test_user, login_and_get_cookie, parse_json_body, test_app,
};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::SqlitePool;
use tower::ServiceExt;
use tracing_test::traced_test;

const WEBHOOK_URL: &str = "https://hooks.waghorn.tech/services/secret-token";

async fn insert_outbox_entry(pool: &SqlitePool, summary: &str, attempts: i64) -> i64 {
    let delivery = PendingDelivery::webhook(
        "Slack",
        WEBHOOK_URL,
        summary,
        &serde_json::json!({"text": summary}),
    )
    .unwrap()
    .for_team(Some(7), Some("Team Rocket"));
    let last_error = (attempts > 0).then_some("webhook returned status 500");
    sqlx::query_scalar(
        "INSERT INTO notification_outbox (delivery, attempts, last_error) VALUES (?, ?, ?) \
         RETURNING id",
    )
    .bind(serde_json::to_string(&delivery).unwrap())
    .bind(attempts)
    .bind(last_error)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn count_outbox_entries(pool: &SqlitePool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM notification_outbox")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outbox_lists_oldest_entries_first_without_delivery_secrets(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let queued_id = insert_outbox_entry(&pool, "Site 'Blog' is back UP", 0).await;
    let failing_id = insert_outbox_entry(&pool, "Site 'Blog' is DOWN", 3).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/notification-outbox")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["total"], 2);
    let entries = body["data"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["id"], queued_id);
    assert_eq!(entries[0]["status"], "queued");
    assert!(entries[0]["last_error"].is_null());
    assert_eq!(entries[1]["id"], failing_id);
    assert_eq!(entries[1]["status"], "failing");
    assert_eq!(entries[1]["provider"], "Slack");
    assert_eq!(entries[1]["summary"], "Site 'Blog' is DOWN");
    assert_eq!(entries[1]["team"], "Team Rocket");
    assert_eq!(entries[1]["team_id"], 7);
    assert_eq!(entries[1]["attempts"], 3);
    assert_eq!(entries[1]["last_error"], "webhook returned status 500");
    assert!(
        !body.to_string().contains("secret-token"),
        "the webhook URL is a secret and must not be listed"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outbox_is_paginated(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let mut ids = Vec::new();
    for _ in 0..25 {
        ids.push(insert_outbox_entry(&pool, "Site 'Blog' is DOWN", 1).await);
    }
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/notification-outbox")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let first_page = parse_json_body(response).await;
    assert_eq!(first_page["total"], 25);
    assert_eq!(first_page["page"], 1);
    assert_eq!(first_page["per_page"], 20);
    assert_eq!(first_page["data"].as_array().unwrap().len(), 20);
    assert_eq!(first_page["data"][0]["id"], ids[0]);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/notification-outbox?page=2")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let second_page = parse_json_body(response).await;
    assert_eq!(second_page["total"], 25);
    assert_eq!(second_page["data"].as_array().unwrap().len(), 5);
    assert_eq!(second_page["data"][0]["id"], ids[20]);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outbox_lists_an_entry_it_cannot_read(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    sqlx::query("INSERT INTO notification_outbox (delivery, attempts) VALUES ('not json', 1)")
        .execute(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/admin/notification-outbox")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["data"][0]["status"], "failing");
    assert!(body["data"][0]["provider"].is_null());
    assert!(body["data"][0]["summary"].is_null());
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_admin_can_cancel_a_failing_entry(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let failing_id = insert_outbox_entry(&pool, "Site 'Blog' is DOWN", 1).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/notification-outbox/{failing_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(count_outbox_entries(&pool).await, 0);
    assert!(logs_contain(
        "Notification cancelled action_by=admin action_by_user_id=1"
    ));
    assert!(logs_contain(
        "notification=Site 'Blog' is DOWN channel=Slack team_id=7 team_name=Team Rocket"
    ));
    assert!(!logs_contain("secret-token"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_entry_that_has_not_failed_cannot_be_cancelled(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let queued_id = insert_outbox_entry(&pool, "Site 'Blog' is DOWN", 0).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/notification-outbox/{queued_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(count_outbox_entries(&pool).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cancelling_a_missing_entry_returns_not_found(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/admin/notification-outbox/999")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outbox_is_admin_only(pool: SqlitePool) {
    insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let failing_id = insert_outbox_entry(&pool, "Site 'Blog' is DOWN", 1).await;
    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/admin/notification-outbox")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/admin/notification-outbox/{failing_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(count_outbox_entries(&pool).await, 1);
}
