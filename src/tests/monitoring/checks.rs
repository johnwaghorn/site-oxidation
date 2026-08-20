use super::*;

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_canary_is_skipped_when_no_sites_are_due(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let site_id = insert_probe_site(
        &pool,
        "Recently Checked",
        server.base_url(),
        200,
        None,
        SiteStatus::Up,
        3600,
        false,
    )
    .await;
    sqlx::query("UPDATE sites SET last_checked_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(site_id)
        .execute(&pool)
        .await
        .unwrap();
    enable_canary(&pool, server.base_url()).await;
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    let checked_at: Option<String> =
        sqlx::query_scalar("SELECT last_checked_at FROM canary_settings WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        checked_at.is_none(),
        "the canary must not run when there is no probe work to protect"
    );
    assert!(logs_contain("No sites due for a probe"));
}

#[test]
fn test_probe_result_is_stale_after_grace_or_backward_clock_change() {
    let started_at = SystemTime::UNIX_EPOCH;
    let expected_duration = Duration::from_secs(30);
    let grace_boundary = started_at.checked_add(Duration::from_secs(35)).unwrap();
    let beyond_grace = started_at
        .checked_add(Duration::from_millis(35_001))
        .unwrap();
    assert!(!probe_result_is_stale(
        started_at,
        grace_boundary,
        expected_duration
    ));
    assert!(probe_result_is_stale(
        started_at,
        beyond_grace,
        expected_duration
    ));
    assert!(probe_result_is_stale(
        grace_boundary,
        started_at,
        expected_duration
    ));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_equivalent_due_sites_with_different_intervals_share_probe(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(
        &pool,
        "Site A",
        &url,
        200,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    insert_probe_site(
        &pool,
        "Site B",
        &url,
        200,
        None,
        SiteStatus::Pending,
        300,
        false,
    )
    .await;
    enable_canary(&pool, &format!("{base_url}/canary")).await;
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 1);
    let updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sites WHERE status = 'up' AND last_checked_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(updated, 2);
    let canary_is_healthy: bool = sqlx::query_scalar(
        "SELECT last_checked_at IS NOT NULL AND last_error IS NULL
         FROM canary_settings WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(canary_is_healthy);
    assert!(logs_contain("Finished checking 2 sites in 1 probes"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_due_site_coalesces_recent_equivalent_monitor_with_same_interval(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(
        &pool,
        "Due",
        &url,
        200,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    let recent_id = insert_probe_site(
        &pool,
        "Recent",
        &url,
        200,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    sqlx::query("UPDATE sites SET last_checked_at = datetime('now') WHERE id = ?")
        .bind(recent_id)
        .execute(&pool)
        .await
        .unwrap();
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 1);
    let updated: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sites WHERE status = 'up' AND last_checked_at IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(updated, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_shared_failure_creates_separate_outages(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(
        &pool,
        "Site A",
        &url,
        503,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    insert_probe_site(
        &pool,
        "Site B",
        &url,
        503,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 1);
    let outages: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(outages, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_shared_failure_retries_when_any_site_was_not_down(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(&pool, "Site A", &url, 503, None, SiteStatus::Up, 60, false).await;
    insert_probe_site(
        &pool,
        "Site B",
        &url,
        503,
        None,
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let mut config = probe_config();
    config.probe_retry_count = 1;
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_shared_failure_does_not_retry_when_all_sites_were_down(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(
        &pool,
        "Site A",
        &url,
        503,
        None,
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    insert_probe_site(
        &pool,
        "Site B",
        &url,
        503,
        None,
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let mut config = probe_config();
    config.probe_retry_count = 1;
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_distinct_probe_keys_do_not_share_probe(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let base_url = server.base_url();
    let url = format!("{base_url}/site");
    insert_probe_site(&pool, "Base", &url, 200, None, SiteStatus::Down, 60, false).await;
    insert_probe_site(
        &pool,
        "Status",
        &url,
        201,
        None,
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    insert_probe_site(
        &pool,
        "Text",
        &url,
        200,
        Some("ok"),
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    insert_probe_site(&pool, "TLS", &url, 200, None, SiteStatus::Down, 60, true).await;
    insert_probe_site(
        &pool,
        "Trailing Slash",
        &format!("{base_url}/site/"),
        200,
        None,
        SiteStatus::Down,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 5);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_grouped_monitors_share_one_notification_per_webhook(pool: SqlitePool) {
    let server = TestHttpServer::start_ignoring_path("/canary").await;
    let dead_port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let url = format!("http://127.0.0.1:{dead_port}/");
    for (team_name, monitor_name) in [("Team Rocket", "Monitor A"), ("Team Aqua", "Monitor B")] {
        let team_id: i64 = sqlx::query_scalar("INSERT INTO teams (name) VALUES (?) RETURNING id")
            .bind(team_name)
            .fetch_one(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO team_notification_settings (team_id, slack_webhook_url) VALUES (?, ?)",
        )
        .bind(team_id)
        .bind(format!("{}/webhook", server.base_url()))
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO sites (
                name, url, expected_status, status,
                probe_interval_seconds, tls_allow_untrusted, team_id
            ) VALUES (?, ?, 200, 'up', 60, 0, ?)",
        )
        .bind(monitor_name)
        .bind(&url)
        .bind(team_id)
        .execute(&pool)
        .await
        .unwrap();
    }
    let client = Client::new();
    let config = probe_config();
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /webhook"));
    assert!(request.contains("is DOWN"));
}
