use crate::notifications::Notifier;
use crate::notifications::delivery::PendingDelivery;
use crate::tests::TestHttpServer;
use reqwest::Client;
use sqlx::SqlitePool;

#[sqlx::test(migrations = "./migrations")]
async fn test_successful_outbox_delivery_is_removed(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let delivery = PendingDelivery::webhook(
        "Slack",
        &format!("{}/webhook", server.base_url()),
        &serde_json::json!({"text": "queued alert"}),
    )
    .unwrap();
    let mut transaction = pool.begin().await.unwrap();
    notifier
        .enqueue(&mut transaction, &[delivery])
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    notifier.process_outbox(&pool).await;
    let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notification_outbox")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(remaining, 0);
    assert_eq!(server.request_count(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_failed_outbox_delivery_is_retained_for_retry(pool: SqlitePool) {
    let dead_port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let notifier = Notifier::new(Client::new(), true);
    let delivery = PendingDelivery::webhook(
        "Slack",
        &format!("http://127.0.0.1:{dead_port}/webhook"),
        &serde_json::json!({"text": "queued alert"}),
    )
    .unwrap();
    let mut transaction = pool.begin().await.unwrap();
    notifier
        .enqueue(&mut transaction, &[delivery])
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    notifier.process_outbox(&pool).await;
    let (attempts, last_error, retry_is_future): (i64, Option<String>, bool) = sqlx::query_as(
        "SELECT attempts, last_error, next_attempt_at > CURRENT_TIMESTAMP \
         FROM notification_outbox",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(attempts, 1);
    assert!(last_error.is_some());
    assert!(retry_is_future);
}
