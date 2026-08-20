use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn test_a_stale_healthy_recheck_is_not_reused(pool: SqlitePool) {
    enable_canary(&pool, "http://127.0.0.1:1").await;
    let client = Client::new();
    let cached_recheck = Mutex::new(None);
    let guard = AmbiguousFailureGuard {
        initial_canary_verdict: CanaryVerdict::Reachable,
        cached_recheck: &cached_recheck,
    };

    *cached_recheck.lock().await = Some(CachedCanaryVerdict {
        completed_at: Instant::now(),
        verdict: CanaryVerdict::Reachable,
    });
    assert!(
        !guard
            .should_suppress_ambiguous_failure(&client, &pool)
            .await,
        "a fresh verdict is reused instead of rechecking"
    );

    *cached_recheck.lock().await = Some(CachedCanaryVerdict {
        completed_at: Instant::now() - CANARY_RECHECK_CACHE_TTL - Duration::from_secs(1),
        verdict: CanaryVerdict::Reachable,
    });
    assert!(
        guard
            .should_suppress_ambiguous_failure(&client, &pool)
            .await,
        "a verdict older than the freshness window must be re-established"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_an_inconclusive_recheck_holds_the_failure_back(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    enable_canary(&pool, server.base_url()).await;
    sqlx::query("DELETE FROM canary_settings WHERE id = 1")
        .execute(&pool)
        .await
        .unwrap();
    let cached_recheck = Mutex::new(None);
    let guard = AmbiguousFailureGuard {
        initial_canary_verdict: CanaryVerdict::Reachable,
        cached_recheck: &cached_recheck,
    };
    assert!(
        guard
            .should_suppress_ambiguous_failure(&Client::new(), &pool)
            .await,
        "an unverifiable failure must not be alerted on"
    );
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_ambiguous_failures_share_one_connectivity_recheck(pool: SqlitePool) {
    let canary_server = TestHttpServer::start().await;
    for port in 1u16..=3 {
        insert_probe_site(
            &pool,
            &format!("Unreachable {port}"),
            &format!("http://127.0.0.1:{port}"),
            200,
            None,
            SiteStatus::Up,
            60,
            false,
        )
        .await;
    }
    let client = Client::new();
    let mut config = probe_config();
    config.probe_timeout_secs = 1;
    enable_canary(&pool, canary_server.base_url()).await;
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    assert_eq!(
        canary_server.request_count(),
        2,
        "three failing groups must share one recheck, not fan out"
    );
    let outage_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        outage_count, 3,
        "the recheck found connectivity healthy, so the failures stand"
    );
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_settings_changed_mid_check_abandons_the_result(pool: SqlitePool) {
    let hanging_server = TestHttpServer::start_hanging().await;
    enable_canary(&pool, hanging_server.base_url()).await;
    let updater = {
        let pool = pool.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            enable_canary(&pool, "http://127.0.0.1:1").await;
        })
    };

    let verdict = run_and_record_canary(&Client::new(), &pool).await;

    updater.await.unwrap();
    assert_eq!(verdict, CanaryVerdict::Inconclusive);
    assert!(logs_contain("Discarding stale canary result"));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_degraded_connectivity_does_not_suppress_a_reachable_failure(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let site_id = insert_probe_site(
        &pool,
        "Wrong Status",
        server.base_url(),
        418,
        None,
        SiteStatus::Up,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let mut config = probe_config();
    config.probe_timeout_secs = 1;
    enable_canary(&pool, "http://127.0.0.1:1").await;
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    let status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
        .bind(site_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let outage_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        status,
        SiteStatus::Down,
        "the server answered, so its failure is not ambiguous"
    );
    assert_eq!(outage_count, 1);
    assert!(logs_contain("Probe connectivity is degraded"));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_degraded_connectivity_persists_successes_and_suppresses_failures(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let successful_site_id = insert_probe_site(
        &pool,
        "Reachable Site",
        server.base_url(),
        200,
        None,
        SiteStatus::Pending,
        60,
        false,
    )
    .await;
    let failed_site_id = insert_probe_site(
        &pool,
        "Ambiguous Failure",
        "http://127.0.0.1:1",
        200,
        None,
        SiteStatus::Up,
        60,
        false,
    )
    .await;
    let client = Client::new();
    let mut config = probe_config();
    config.probe_timeout_secs = 1;
    enable_canary(&pool, "http://127.0.0.1:1").await;
    run_due_site_checks(
        &client,
        &client,
        &pool,
        &config,
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await;
    let successful_status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
        .bind(successful_site_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let failed_status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
        .bind(failed_site_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let outage_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(successful_status, SiteStatus::Up);
    assert_eq!(failed_status, SiteStatus::Up);
    assert_eq!(outage_count, 0);
    assert!(logs_contain("Suppressing unreachable probe result"));
}
