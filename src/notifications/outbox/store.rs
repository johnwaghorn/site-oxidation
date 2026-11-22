use crate::notifications::delivery::PendingDelivery;
use jiff::Timestamp;
use sqlx::{FromRow, Sqlite, SqlitePool, Transaction};

#[derive(FromRow)]
pub(super) struct OutboxRow {
    pub(super) id: i64,
    pub(super) delivery: String,
    pub(super) attempts: i64,
}

#[derive(FromRow)]
struct StoredEntry {
    id: i64,
    delivery: String,
    attempts: i64,
    last_error: Option<String>,
    created_at: jiff_sqlx::Timestamp,
    next_attempt_at: jiff_sqlx::Timestamp,
}

pub(crate) struct OutboxEntry {
    pub id: i64,
    pub provider: Option<String>,
    pub summary: Option<String>,
    pub team_id: Option<i64>,
    pub team: Option<String>,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub created_at: Timestamp,
    pub next_attempt_at: Timestamp,
}

impl OutboxEntry {
    pub(crate) fn is_failing(&self) -> bool {
        self.attempts > 0
    }
}

impl From<StoredEntry> for OutboxEntry {
    fn from(stored: StoredEntry) -> Self {
        let description = NotificationDescription::read_from_stored_delivery(&stored.delivery);
        Self {
            id: stored.id,
            provider: description.provider,
            summary: description.summary,
            team_id: description.team_id,
            team: description.team,
            attempts: stored.attempts,
            last_error: stored.last_error,
            created_at: stored.created_at.to_jiff(),
            next_attempt_at: stored.next_attempt_at.to_jiff(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct NotificationDescription {
    pub provider: Option<String>,
    pub summary: Option<String>,
    pub team_id: Option<i64>,
    pub team: Option<String>,
}

impl NotificationDescription {
    fn read_from_stored_delivery(stored_delivery: &str) -> Self {
        serde_json::from_str::<PendingDelivery>(stored_delivery).map_or_default(|delivery| Self {
            provider: Some(delivery.provider),
            summary: delivery.summary,
            team_id: delivery.team_id,
            team: delivery.team,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CancelOutcome {
    Cancelled(NotificationDescription),
    BeingSent,
    NotFailing,
    NotFound,
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
         WHERE next_attempt_at <= strftime('%Y-%m-%dT%H:%M:%SZ', 'now') ORDER BY id LIMIT ?",
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
         next_attempt_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now', ?) WHERE id = ?",
    )
    .bind(attempts)
    .bind(error)
    .bind(retry_modifier)
    .bind(row.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub(crate) async fn list_entries_oldest_first(
    pool: &SqlitePool,
    limit: u32,
    offset: u32,
) -> sqlx::Result<Vec<OutboxEntry>> {
    let stored = sqlx::query_as::<_, StoredEntry>(
        "SELECT id, delivery, attempts, last_error, created_at, next_attempt_at \
         FROM notification_outbox ORDER BY id LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(stored.into_iter().map(OutboxEntry::from).collect())
}

pub(crate) async fn count_entries(pool: &SqlitePool) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM notification_outbox")
        .fetch_one(pool)
        .await
}

pub(in crate::notifications) async fn is_queued(pool: &SqlitePool, id: i64) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM notification_outbox WHERE id = ?)")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub(in crate::notifications) async fn cancel_failing_entry(
    pool: &SqlitePool,
    id: i64,
) -> sqlx::Result<CancelOutcome> {
    let cancelled_delivery: Option<String> = sqlx::query_scalar(
        "DELETE FROM notification_outbox WHERE id = ? AND attempts > 0 RETURNING delivery",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    if let Some(cancelled_delivery) = cancelled_delivery {
        return Ok(CancelOutcome::Cancelled(
            NotificationDescription::read_from_stored_delivery(&cancelled_delivery),
        ));
    }
    Ok(if is_queued(pool, id).await? {
        CancelOutcome::NotFailing
    } else {
        CancelOutcome::NotFound
    })
}
