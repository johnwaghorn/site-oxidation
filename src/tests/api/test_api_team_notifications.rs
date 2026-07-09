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
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    assert_eq!(body["smtp_tls_mode"], "starttls");
    assert_eq!(body["smtp_auth"], true);
    assert_eq!(body["smtp_password_set"], false);
    assert_eq!(body["notify_site_down"], true);
    assert_eq!(body["notify_site_recovered"], true);
    assert_eq!(body["notify_cert_expiring"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_can_update_non_slack_notification_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
                        "smtp_tls_mode":"tls",
                        "smtp_auth":false,
                        "smtp_username":" alerts@waghorn.tech ",
                        "smtp_password":" secret ",
                        "smtp_from_email":" alerts@waghorn.tech ",
                        "smtp_to_email":" john@waghorn.tech ",
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
    assert_eq!(body["smtp_tls_mode"], "tls");
    assert_eq!(body["smtp_auth"], false);
    assert_eq!(body["smtp_username"], "alerts@waghorn.tech");
    assert_eq!(body["smtp_from_email"], "alerts@waghorn.tech");
    assert_eq!(body["smtp_to_email"], "john@waghorn.tech");
    assert_eq!(body["smtp_password_set"], true);
    assert_eq!(body["notify_site_down"], true);
    assert_eq!(body["notify_site_recovered"], false);
    assert_eq!(body["notify_cert_expiring"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_partial_update_preserves_existing_notification_settings(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
            smtp_host, smtp_port, smtp_tls_mode, smtp_auth, smtp_username, smtp_password,
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
    .bind("john@waghorn.tech")
    .bind(true)
    .bind(false)
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    assert_eq!(body["smtp_tls_mode"], "tls");
    assert_eq!(body["smtp_auth"], false);
    assert_eq!(body["notify_site_recovered"], false);
    assert_eq!(body["notify_cert_expiring"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_null_fields_are_ignored(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
async fn test_partial_smtp_config_is_rejected_and_rolled_back(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"smtp_host":"smtp.waghorn.tech"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["message"],
        "SMTP host, from address and to address must be saved together"
    );

    let saved_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM team_notification_settings WHERE team_id = ?")
            .bind(team_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(saved_rows, 0, "the rejected upsert should be rolled back");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_smtp_auth_requires_credentials(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{
                        "smtp_host":"smtp.waghorn.tech",
                        "smtp_from_email":"alerts@waghorn.tech",
                        "smtp_to_email":"john@waghorn.tech",
                        "smtp_auth":true
                    }"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["message"],
        "SMTP authentication is enabled but the username or password is missing"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_non_smtp_update_skips_validation_of_existing_partial_config(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query("INSERT INTO team_notification_settings (team_id, smtp_host) VALUES (?, 'smtp.waghorn.tech')")
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

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/teams/{team_id}/notifications"))
                .header("cookie", &cookie)
                .header("content-type", "application/json")
                .body(Body::from(r#"{"notify_site_down":false}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["notify_site_down"], false);
    assert_eq!(body["smtp_host"], "smtp.waghorn.tech");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_telegram_settings_must_be_updated_together(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
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

#[sqlx::test(migrations = "./migrations")]
async fn test_test_email_requires_configured_smtp(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/email"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_test_email_reports_send_failure(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let dead_port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    sqlx::query(
        "INSERT INTO team_notification_settings (
            team_id, smtp_host, smtp_port, smtp_tls_mode, smtp_auth, smtp_from_email, smtp_to_email
        ) VALUES (?, '127.0.0.1', ?, 'none', 0, 'alerts@waghorn.tech', 'john@waghorn.tech')",
    )
    .bind(team_id)
    .bind(i64::from(dead_port))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/email"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body = parse_json_body(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .starts_with("Test email failed:")
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_test_slack_requires_configuration(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
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
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/slack"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_test_slack_posts_to_configured_webhook(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let server = crate::tests::TestHttpServer::start().await;
    sqlx::query(
        "INSERT INTO team_notification_settings (team_id, slack_webhook_url) VALUES (?, ?)",
    )
    .bind(team_id)
    .bind(format!("{}/slack", server.base_url()))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/slack"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /slack"));
    assert!(request.contains("Testing Site Oxidation notification configuration"));
    assert!(request.contains("https://github.com/johnwaghorn/site-oxidation"));
    assert!(request.contains("Triggered by: maddie"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_test_teams_posts_adaptive_card(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    let server = crate::tests::TestHttpServer::start().await;
    sqlx::query(
        "INSERT INTO team_notification_settings (team_id, microsoft_teams_webhook_url) VALUES (?, ?)",
    )
    .bind(team_id)
    .bind(format!("{}/teams", server.base_url()))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO team_members (team_id, user_id) VALUES (?, ?)")
        .bind(team_id)
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/teams"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /teams"));
    assert!(request.contains("application/vnd.microsoft.card.adaptive"));
    assert!(request.contains("Testing Site Oxidation notification configuration"));
    assert!(request.contains("Triggered by"));
    assert!(request.contains("maddie"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_test_email_blocks_private_hosts_by_default(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    let team_id: i64 =
        sqlx::query_scalar("INSERT INTO teams (name) VALUES ('Team Rocket') RETURNING id")
            .fetch_one(&pool)
            .await
            .unwrap();
    sqlx::query(
        "INSERT INTO team_notification_settings (
            team_id, smtp_host, smtp_tls_mode, smtp_auth, smtp_from_email, smtp_to_email
        ) VALUES (?, '127.0.0.1', 'none', 0, 'alerts@waghorn.tech', 'john@waghorn.tech')",
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
    let app = crate::tests::test_app_with_private_ips(pool, false);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/teams/{team_id}/notifications/test/email"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body = parse_json_body(response).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("private address")
    );
}
