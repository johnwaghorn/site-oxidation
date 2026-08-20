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
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
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
