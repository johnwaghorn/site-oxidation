use crate::models::site::SiteStatus;
use crate::tests::{
    TEST_PASSWORD, TEST_SITE_NAME, authenticated_admin_app, insert_team_sites, insert_test_site,
    insert_test_user, login_and_get_cookie, parse_json_body, test_app,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn insert_site_added_a_day_ago_with_outage(pool: &SqlitePool, outage_seconds: i64) -> i64 {
    let site_id: i64 = sqlx::query_scalar(
        "INSERT INTO sites (name, url, status, created_at, last_checked_at) \
         VALUES ('Day Old', 'https://day-old.waghorn.tech', 'up', \
                 datetime('now', '-1 day'), datetime('now')) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO outages (site_id, started_at, ended_at) \
         VALUES (?1, datetime('now', '-2 hours'), datetime('now', '-2 hours', ?2))",
    )
    .bind(site_id)
    .bind(format!("+{outage_seconds} seconds"))
    .execute(pool)
    .await
    .unwrap();
    site_id
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_returns_one_day_per_entry_ending_today(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/sites/{}/uptime?days=7&time_zone=Europe/London",
                    site.id
                ))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let days = body["days"].as_array().unwrap();
    assert_eq!(days.len(), 7);
    let today_in_london = jiff::Zoned::now()
        .in_tz("Europe/London")
        .unwrap()
        .date()
        .to_string();
    assert_eq!(days[6]["date"], today_in_london);
    assert!(body["down_since"].is_null());
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_is_empty_until_the_site_has_been_checked(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Pending).await;
    sqlx::query("UPDATE sites SET created_at = datetime('now', '-1 hour') WHERE id = ?")
        .bind(site.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO monitoring_periods (started_at, last_seen_at) \
         VALUES (datetime('now', '-2 hours'), datetime('now'))",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/sites/{}/uptime?days=7&time_zone=UTC", site.id))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert!(
        body["uptime_basis_points"].is_null(),
        "nothing has been checked, got {}",
        body["uptime_basis_points"]
    );
    assert!(body["days"][6]["uptime_basis_points"].is_null());
    assert_eq!(body["days"][6]["monitoring_started"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_reports_an_ongoing_outage(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query(
        "UPDATE sites SET created_at = datetime('now', '-1 hour'), last_checked_at = datetime('now') \
         WHERE id = ?",
    )
    .bind(site.id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO outages (site_id, started_at) VALUES (?, datetime('now', '-5 minutes'))",
    )
    .bind(site.id)
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/sites/{}/uptime?days=7&time_zone=UTC", site.id))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert!(body["down_since"].is_string());
    assert_eq!(body["days"][6]["outage_count"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_rejects_unsupported_number_of_days(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/sites/{}/uptime?days=10&time_zone=UTC", site.id))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(body["message"], "days must be one of 7, 30, 90");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_rejects_unknown_time_zone(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/sites/{}/uptime?days=7&time_zone=Mars/Olympus_Mons",
                    site.id
                ))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = parse_json_body(response).await;
    assert_eq!(body["message"], "Unknown time zone");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_another_teams_site_uptime_is_not_found(pool: SqlitePool) {
    let user_id = insert_test_user(&pool, "maddie", TEST_PASSWORD, "user", false).await;
    insert_team_sites(&pool, user_id).await;
    let other_team_site_id: i64 = sqlx::query_scalar("SELECT id FROM sites WHERE name = 'Shop'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let app = test_app(pool);
    let cookie = login_and_get_cookie(&app, "maddie", TEST_PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/sites/{other_team_site_id}/uptime?days=7&time_zone=UTC"
                ))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_list_sites_includes_uptime_for_each_checked_site(pool: SqlitePool) {
    insert_site_added_a_day_ago_with_outage(&pool, 864).await;
    insert_test_site(&pool, SiteStatus::Pending).await;
    sqlx::query(
        "INSERT INTO monitoring_periods (started_at, last_seen_at) \
         VALUES (datetime('now', '-2 days'), datetime('now'))",
    )
    .execute(&pool)
    .await
    .unwrap();
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
    let uptimes: Vec<(&str, Option<i64>, Option<i64>)> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| {
            (
                site["name"].as_str().unwrap(),
                site["uptime_basis_points"].as_i64(),
                site["uptime_window_days"].as_i64(),
            )
        })
        .collect();
    assert_eq!(
        uptimes,
        [
            (TEST_SITE_NAME, None, Some(90)),
            ("Day Old", Some(9900), Some(90))
        ],
        "864s down in a day is 99.00%; a site never checked has no uptime; both say the figure covers 90 days"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_figure_only_covers_the_last_90_days(pool: SqlitePool) {
    let site_id: i64 = sqlx::query_scalar(
        "INSERT INTO sites (name, url, status, created_at, last_checked_at) \
         VALUES ('Year Old', 'https://year-old.waghorn.tech', 'up', \
                 datetime('now', '-365 days'), datetime('now')) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO outages (site_id, started_at, ended_at) VALUES \
         (?1, datetime('now', '-200 days'), datetime('now', '-100 days')), \
         (?1, datetime('now', '-9 days'), datetime('now', '-216 hours', '+7776 seconds'))",
    )
    .bind(site_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO monitoring_periods (started_at, last_seen_at) \
         VALUES (datetime('now', '-365 days'), datetime('now'))",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/sites/{site_id}"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    assert_eq!(
        body["uptime_basis_points"], 9990,
        "7776s down in 90 days is 99.90%; the outage that ended 100 days ago is left out"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_uptime_leaves_out_time_the_monitor_was_not_running(pool: SqlitePool) {
    let site_id = insert_site_added_a_day_ago_with_outage(&pool, 864).await;
    sqlx::query(
        "INSERT INTO monitoring_periods (started_at, last_seen_at) \
         VALUES (datetime('now', '-1 day'), datetime('now', '-3 hours'))",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (app, cookie) = authenticated_admin_app(pool, true).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/sites/{site_id}/uptime?days=7&time_zone=UTC"))
                .header("cookie", &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json_body(response).await;
    let unwatched_seconds = body["unwatched_seconds"].as_i64().unwrap();
    assert!(
        (3 * 3600..3 * 3600 + 5).contains(&unwatched_seconds),
        "the last 3 hours had no heartbeat, got {unwatched_seconds}"
    );
    assert_eq!(
        body["uptime_basis_points"], 10_000,
        "the outage 2 hours ago fell in unwatched time"
    );
}
