use crate::models::site::SiteStatus;
use crate::tests::{
    TEST_PROBE_INTERVAL_SECONDS, TEST_SITE_NAME, TEST_SITE_URL, authenticated_admin_app,
    insert_test_site, parse_json_body,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_returns_inserted_site(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
    assert!(sites[0]["name"].as_str().unwrap().contains(TEST_SITE_NAME));
    assert!(!sites[0]["created_at"].as_str().unwrap().is_empty());
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 20);
    assert_eq!(body["total"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_accepts_frontend_pagination_query(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites?page=1&per_page=20")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_filters_by_name_or_url_and_ignores_blank_search(pool: SqlitePool) {
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status) VALUES \
         ('Documentation', 'https://docs.waghorn.tech', 200, 'up'), \
         ('Status Page', 'https://status.waghorn.tech', 200, 'up')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;

    for (search, expected_name) in [
        ("document", "Documentation"),
        ("status.waghorn", "Status Page"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/sites?page=1&per_page=20&search={search}"))
                    .header("cookie", &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = parse_json_body(response).await;
        assert_eq!(body["total"], 1);
        assert_eq!(body["data"][0]["name"], expected_name);
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites?search=%20%20")
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
async fn test_list_sites_filters_by_status_and_down_includes_blocked(pool: SqlitePool) {
    sqlx::query(
        "INSERT INTO sites (name, url, expected_status, status) VALUES \
         ('Up Site', 'https://up.waghorn.tech', 200, 'up'), \
         ('Down Site', 'https://down.waghorn.tech', 200, 'down'), \
         ('Blocked Site', 'https://blocked.waghorn.tech', 200, 'blocked'), \
         ('Pending Site', 'https://pending.waghorn.tech', 200, 'pending')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/sites?status=up")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(body["total"], 1, "only the up site should match status=up");
    assert_eq!(body["data"][0]["name"], "Up Site");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/sites?status=down")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["total"], 2,
        "status=down should match down and blocked sites"
    );
    let names: Vec<&str> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"Down Site"),
        "down site missing from {names:?}"
    );
    assert!(
        names.contains(&"Blocked Site"),
        "blocked site missing from {names:?}"
    );

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
    assert_eq!(
        body["total"], 4,
        "no status filter should return every site"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_rejects_unknown_status_filter(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri("/sites?status=pending")
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
    let body = parse_json_body(response).await;
    assert!(!body["created_at"].as_str().unwrap().is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_duplicate_url_returns_409(pool: SqlitePool) {
    insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let payload = format!(
        r#"{{"name":"A Different Name","url":"{TEST_SITE_URL}","probe_interval_seconds":{TEST_PROBE_INTERVAL_SECONDS}}}"#,
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
    let body = parse_json_body(response).await;
    assert_eq!(
        body["message"],
        "A site with that URL already exists for this team"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_create_site_invalid_payload_returns_422(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
    let (app, cookie) = authenticated_admin_app(pool, false).await;
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
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
    let (app, cookie) = authenticated_admin_app(pool, false).await;
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
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
    let (app, cookie) = authenticated_admin_app(pool.clone(), true).await;
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
async fn test_admin_can_create_site_without_team_id(pool: SqlitePool) {
    let (app, cookie) = authenticated_admin_app(pool, true).await;
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
