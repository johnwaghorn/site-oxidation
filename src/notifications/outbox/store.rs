use crate::notifications::delivery::PendingDelivery;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};

#[derive(FromRow)]
pub(super) struct OutboxRow {
    pub(super) id: i64,
    pub(super) delivery: String,
    pub(super) attempts: i64,
}

pub(in crate::notifications) async fn enqueue(
    transaction: &mut Transaction<'_, Sqlite>,
    deliveries: &[PendingDelivery],
) -> anyhow::Result<()> {
    for delivery in deliveries {
        let serialized = serde_json::to_string(delivery)?;
        sqlx::query("INSERT INTO notification_outbox (delivery) VALUES (?)")
            .bind(serialized)
            .execute(&mut **transaction)
            .await?;
    }
    Ok(())
}

pub(super) async fn load_due(pool: &SqlitePool, limit: i64) -> sqlx::Result<Vec<OutboxRow>> {
    sqlx::query_as::<_, OutboxRow>(
        "SELECT id, delivery, attempts FROM notification_outbox \
         WHERE next_attempt_at <= CURRENT_TIMESTAMP ORDER BY id LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub(super) async fn delete(pool: &SqlitePool, id: i64) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM notification_outbox WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub(super) async fn record_failure(
    pool: &SqlitePool,
    row: &OutboxRow,
    error: &str,
    delay_seconds: u64,
) -> sqlx::Result<()> {
    let attempts = row.attempts.saturating_add(1);
    let retry_modifier = format!("+{delay_seconds} seconds");
    let error: String = error.chars().take(500).collect();
    sqlx::query(
        "UPDATE notification_outbox SET attempts = ?, last_error = ?, \
         next_attempt_at = datetime('now', ?) WHERE id = ?",
    )
    .bind(attempts)
    .bind(error)
    .bind(retry_modifier)
    .bind(row.id)
    .execute(pool)
    .await?;
    Ok(())
}
