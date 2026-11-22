use crate::notifications::Notifier;
use crate::notifications::delivery::PendingDelivery;
use crate::notifications::outbox::CancelOutcome;
use crate::tests::TestHttpServer;
use reqwest::Client;
use sqlx::SqlitePool;
use std::time::Duration;

async fn enqueue_failing_webhook(pool: &SqlitePool, notifier: &Notifier, url: &str) -> i64 {
    let delivery = PendingDelivery::webhook(
        "Slack",
        url,
        "queued alert",
        &serde_json::json!({"text": "queued alert"}),
    )
    .unwrap();
    let mut transaction = pool.begin().await.unwrap();
    notifier
        .enqueue(&mut transaction, &[delivery])
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    sqlx::query_scalar(
        "UPDATE notification_outbox SET attempts = 1 \
         WHERE id = (SELECT MAX(id) FROM notification_outbox) RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn wait_for_first_request(server: &TestHttpServer) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while server.request_count() == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the outbox worker never started sending");
}

#[sqlx::test(migrations = "./migrations")]
async fn test_successful_outbox_delivery_is_removed(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let delivery = PendingDelivery::webhook(
        "Slack",
        &format!("{}/webhook", server.base_url()),
        "queued alert",
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
        "queued alert",
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
        "SELECT attempts, last_error, next_attempt_at > strftime('%Y-%m-%dT%H:%M:%SZ', 'now') \
         FROM notification_outbox",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(attempts, 1);
    assert!(last_error.is_some());
    assert!(retry_is_future);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_entry_cancelled_while_the_worker_is_busy_is_not_sent(pool: SqlitePool) {
    let stuck_server = TestHttpServer::start_hanging().await;
    let second_server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    enqueue_failing_webhook(&pool, &notifier, stuck_server.base_url()).await;
    let second_id = enqueue_failing_webhook(&pool, &notifier, second_server.base_url()).await;
    let worker = tokio::spawn({
        let notifier = notifier.clone();
        let pool = pool.clone();
        async move { notifier.process_outbox(&pool).await }
    });
    wait_for_first_request(&stuck_server).await;

    let outcome = notifier
        .cancel_failing_outbox_entry(&pool, second_id)
        .await
        .unwrap();
    assert!(matches!(outcome, CancelOutcome::Cancelled(_)));
    drop(stuck_server);
    worker.await.unwrap();

    assert_eq!(
        second_server.request_count(),
        0,
        "the worker loaded this entry before it was cancelled, but must not send it"
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn test_entry_being_sent_cannot_be_cancelled(pool: SqlitePool) {
    let stuck_server = TestHttpServer::start_hanging().await;
    let notifier = Notifier::new(Client::new(), true);
    let id = enqueue_failing_webhook(&pool, &notifier, stuck_server.base_url()).await;
    let worker = tokio::spawn({
        let notifier = notifier.clone();
        let pool = pool.clone();
        async move { notifier.process_outbox(&pool).await }
    });
    wait_for_first_request(&stuck_server).await;

    let outcome = notifier
        .cancel_failing_outbox_entry(&pool, id)
        .await
        .unwrap();
    assert_eq!(outcome, CancelOutcome::BeingSent);
    drop(stuck_server);
    worker.await.unwrap();

    let attempts: i64 = sqlx::query_scalar("SELECT attempts FROM notification_outbox WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        attempts, 2,
        "the entry stays queued and its failed send is recorded"
    );
    let outcome = notifier
        .cancel_failing_outbox_entry(&pool, id)
        .await
        .unwrap();
    assert!(matches!(outcome, CancelOutcome::Cancelled(_)));
}
