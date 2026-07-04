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
async fn test_team_member_can_manage_slack_webhook(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"slack_webhook_url":" https://hooks.slack.test/services/team-rocket "}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["team_id"], team_id);
    assert_eq!(
        body["slack_webhook_url"],
        "https://hooks.slack.test/services/team-rocket"
    );

    let saved: Option<String> = sqlx::query_scalar(
        "SELECT slack_webhook_url FROM team_notification_settings WHERE team_id = ?",
    )
    .bind(team_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        saved.as_deref(),
        Some("https://hooks.slack.test/services/team-rocket")
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["slack_webhook_url"],
        "https://hooks.slack.test/services/team-rocket"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_blank_slack_webhook_clears_value(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query(
        "INSERT INTO team_notification_settings (team_id, slack_webhook_url) VALUES (?, 'https://hooks.slack.test/services/team-rocket')",
    )
    .bind(team_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slack_webhook_url":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert!(body["slack_webhook_url"].is_null());

    let saved: Option<String> = sqlx::query_scalar(
        "SELECT slack_webhook_url FROM team_notification_settings WHERE team_id = ?",
    )
    .bind(team_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(saved, None);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_get_notifications_returns_defaults_without_settings_row(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["team_id"], team_id);
    assert!(body["slack_webhook_url"].is_null());
    assert!(body["microsoft_teams_webhook_url"].is_null());
    assert_eq!(body["telegram_bot_token_set"], false);
    assert!(body["telegram_chat_id"].is_null());
    assert_eq!(body["smtp_security"], "starttls");
    assert_eq!(body["smtp_auth"], true);
    assert_eq!(body["smtp_password_set"], false);
    assert_eq!(body["notify_site_down"], true);
    assert_eq!(body["notify_site_recovered"], true);
    assert_eq!(body["notify_cert_expiring"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_can_update_non_slack_notification_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "microsoft_teams_webhook_url":" https://teams.waghorn.tech/webhook ",
                        "telegram_bot_token":" token-123 ",
                        "telegram_chat_id":" 12345 ",
                        "smtp_host":" smtp.waghorn.tech ",
                        "smtp_port":587,
                        "smtp_security":"tls",
                        "smtp_auth":false,
                        "smtp_username":" alerts@waghorn.tech ",
                        "smtp_password":" secret ",
                        "smtp_from_email":" alerts@waghorn.tech ",
                        "smtp_to_email":" on-call@waghorn.tech ",
                        "notify_site_down":true,
                        "notify_site_recovered":false,
                        "notify_cert_expiring":false
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["microsoft_teams_webhook_url"],
        "https://teams.waghorn.tech/webhook"
    );
    assert_eq!(body["telegram_bot_token_set"], true);
    assert_eq!(body["telegram_chat_id"], "12345");
    assert_eq!(body["smtp_host"], "smtp.waghorn.tech");
    assert_eq!(body["smtp_port"], 587);
    assert_eq!(body["smtp_security"], "tls");
    assert_eq!(body["smtp_auth"], false);
    assert_eq!(body["smtp_username"], "alerts@waghorn.tech");
    assert_eq!(body["smtp_from_email"], "alerts@waghorn.tech");
    assert_eq!(body["smtp_to_email"], "on-call@waghorn.tech");
    assert_eq!(body["smtp_password_set"], true);
    assert_eq!(body["notify_site_down"], true);
    assert_eq!(body["notify_site_recovered"], false);
    assert_eq!(body["notify_cert_expiring"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_partial_update_preserves_existing_notification_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO team_notification_settings (
            team_id, microsoft_teams_webhook_url, telegram_bot_token, telegram_chat_id,
            smtp_host, smtp_port, smtp_security, smtp_auth, smtp_username, smtp_password,
            smtp_from_email, smtp_to_email, notify_site_down, notify_site_recovered, notify_cert_expiring
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(team_id)
    .bind("https://teams.waghorn.tech/webhook")
    .bind("token-123")
    .bind("12345")
    .bind("smtp.waghorn.tech")
    .bind(587)
    .bind("tls")
    .bind(false)
    .bind("alerts@waghorn.tech")
    .bind("secret")
    .bind("alerts@waghorn.tech")
    .bind("on-call@waghorn.tech")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"slack_webhook_url":"https://hooks.slack.test/services/team-rocket"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["slack_webhook_url"],
        "https://hooks.slack.test/services/team-rocket"
    );
    assert_eq!(
        body["microsoft_teams_webhook_url"],
        "https://teams.waghorn.tech/webhook"
    );
    assert_eq!(body["telegram_bot_token_set"], true);
    assert_eq!(body["telegram_chat_id"], "12345");
    assert_eq!(body["smtp_security"], "tls");
    assert_eq!(body["smtp_auth"], false);
    assert_eq!(body["notify_site_recovered"], false);
    assert_eq!(body["notify_cert_expiring"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_null_fields_are_ignored(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query(
        "INSERT INTO team_notification_settings (team_id, slack_webhook_url, smtp_host, smtp_port) VALUES (?, 'https://hooks.slack.test/services/team-rocket', 'smtp.waghorn.tech', 587)",
    )
    .bind(team_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool.clone());
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slack_webhook_url":null,"smtp_port":null}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["slack_webhook_url"],
        "https://hooks.slack.test/services/team-rocket"
    );
    assert_eq!(body["smtp_port"], 587);
    assert_eq!(body["smtp_host"], "smtp.waghorn.tech");

    let saved: Option<i64> =
        sqlx::query_scalar("SELECT smtp_port FROM team_notification_settings WHERE team_id = ?")
            .bind(team_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(saved, Some(587));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_telegram_settings_must_be_updated_together(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"telegram_bot_token":"token-123"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_member_cannot_manage_team_slack_webhook(pool: SqlitePool) {
    insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"slack_webhook_url":"https://hooks.slack.test/services/team-rocket"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_admin_can_manage_any_team_slack_webhook(pool: SqlitePool) {
    insert_test_user(&pool, "admin", TEST_PASSWORD, "admin", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "admin", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"slack_webhook_url":"https://hooks.slack.test/services/team-rocket"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["slack_webhook_url"],
        "https://hooks.slack.test/services/team-rocket"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_invalid_slack_webhook_url_is_rejected(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "alice", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "alice", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"slack_webhook_url":"not a url"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
