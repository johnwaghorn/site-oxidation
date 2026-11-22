use super::*;

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_outage_created_when_site_goes_down(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_down_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
        .bind(site.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
    let expected_status: Option<i64> =
        sqlx::query_scalar("SELECT expected_status FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(expected_status, Some(site.expected_status));
    assert!(logs_contain("Site 'Waghorn Technology Ltd' is DOWN"));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_outage_closed_when_site_recovers(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query("INSERT INTO outages (site_id, http_status, error_message) VALUES (?, ?, ?)")
        .bind(site.id)
        .bind(500)
        .bind(String::from("Server cooked"))
        .execute(&pool)
        .await
        .unwrap();
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_up_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    let outage_ended: Option<String> =
        sqlx::query_scalar("SELECT ended_at FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(outage_ended.is_some());
    assert!(logs_contain("Site 'Waghorn Technology Ltd' is back UP"));
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_blocked_transition_is_logged_with_reason(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_blocked_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    let status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
        .bind(site.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, SiteStatus::Blocked);
    assert!(logs_contain(&format!(
        "Site 'Waghorn Technology Ltd' probe is BLOCKED - {PRIVATE_IP_BLOCKED_MESSAGE}"
    )));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_blocked_period_is_recorded_while_the_site_is_blocked(pool: SqlitePool) {
    let mut site = insert_test_site(&pool, SiteStatus::Up).await;
    let notifier = Notifier::new(Client::new(), test_config(true).smtp_allow_private_hosts);
    let blocked_periods = || async {
        sqlx::query_as::<_, (i64, bool)>(
            "SELECT COUNT(*), COALESCE(SUM(ended_at IS NULL), 0) > 0 FROM site_blocked_periods \
             WHERE site_id = ?",
        )
        .bind(site.id)
        .fetch_one(&pool)
        .await
        .unwrap()
    };

    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_blocked_result(),
        &notifier,
    )
    .await
    .unwrap();
    assert_eq!(blocked_periods().await, (1, true), "one open period");

    site.status = SiteStatus::Blocked;
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_blocked_result(),
        &notifier,
    )
    .await
    .unwrap();
    assert_eq!(blocked_periods().await, (1, true), "still the same period");

    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_up_result(),
        &notifier,
    )
    .await
    .unwrap();
    assert_eq!(blocked_periods().await, (1, false), "closed once unblocked");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_blocked_on_its_first_check_is_blocked_from_when_it_was_added(pool: SqlitePool) {
    let never_checked = insert_test_site(&pool, SiteStatus::Pending).await;
    sqlx::query("UPDATE sites SET created_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 hour') WHERE id = ?")
        .bind(never_checked.id)
        .execute(&pool)
        .await
        .unwrap();
    let notifier = Notifier::new(Client::new(), test_config(true).smtp_allow_private_hosts);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&never_checked),
        &mock_site_blocked_result(),
        &notifier,
    )
    .await
    .unwrap();
    let starts_when_added: bool = sqlx::query_scalar(
        "SELECT datetime(p.started_at) = datetime(s.created_at) FROM site_blocked_periods p \
         JOIN sites s ON s.id = p.site_id WHERE s.id = ?",
    )
    .bind(never_checked.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(starts_when_added);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_blocked_after_working_is_blocked_from_now(pool: SqlitePool) {
    let working = insert_test_site(&pool, SiteStatus::Up).await;
    sqlx::query("UPDATE sites SET created_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 hour') WHERE id = ?")
        .bind(working.id)
        .execute(&pool)
        .await
        .unwrap();
    let notifier = Notifier::new(Client::new(), test_config(true).smtp_allow_private_hosts);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&working),
        &mock_site_blocked_result(),
        &notifier,
    )
    .await
    .unwrap();
    let started_just_now: bool = sqlx::query_scalar(
        "SELECT started_at >= strftime('%Y-%m-%dT%H:%M:%SZ', 'now', '-1 minute') FROM site_blocked_periods \
         WHERE site_id = ?",
    )
    .bind(working.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(started_just_now);
}

#[sqlx::test(migrations = "./migrations")]
#[traced_test]
async fn test_blocked_transition_closes_open_outage(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Down).await;
    sqlx::query("INSERT INTO outages (site_id) VALUES (?)")
        .bind(site.id)
        .execute(&pool)
        .await
        .unwrap();
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_blocked_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    let outage_ended: Option<String> =
        sqlx::query_scalar("SELECT ended_at FROM outages WHERE site_id = ?")
            .bind(site.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(outage_ended.is_some());
    assert!(logs_contain(
        "Site 'Waghorn Technology Ltd' probe is BLOCKED"
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_no_duplicate_outage_when_already_down(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Down).await;
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_down_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    assert_eq!(count_outages(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_status_update_rolls_back_when_outage_insert_fails(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Up).await;
    sqlx::query("INSERT INTO outages (site_id) VALUES (?)")
        .bind(site.id)
        .execute(&pool)
        .await
        .unwrap();
    let mut transaction = pool.begin().await.unwrap();
    let result = update_site_status(&mut transaction, &site, &mock_site_down_result()).await;
    assert!(result.is_err());
    transaction.rollback().await.unwrap();
    let status: SiteStatus = sqlx::query_scalar("SELECT status FROM sites WHERE id = ?")
        .bind(site.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, SiteStatus::Up);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_outage_created_when_pending_site_goes_down(pool: SqlitePool) {
    let site = insert_test_site(&pool, SiteStatus::Pending).await;
    let config = test_config(true);
    persist_site_statuses(
        &pool,
        std::slice::from_ref(&site),
        &mock_site_down_result(),
        &Notifier::new(Client::new(), config.smtp_allow_private_hosts),
    )
    .await
    .unwrap();
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages WHERE site_id = ?")
        .bind(site.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);
}
