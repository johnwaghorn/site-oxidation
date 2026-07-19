use super::store::{self, OutboxRow};
use crate::notifications::delivery::PendingDelivery;
use crate::notifications::sender;
use reqwest::Client;
use sqlx::SqlitePool;

const BATCH_SIZE: i64 = 100;
const INITIAL_RETRY_SECONDS: u64 = 30;
const MAX_RETRY_SECONDS: u64 = 3600;

pub(in crate::notifications) async fn process(
    pool: &SqlitePool,
    client: &Client,
    allow_private_smtp_hosts: bool,
) -> anyhow::Result<()> {
    for row in store::load_due(pool, BATCH_SIZE).await? {
        let delivery = match serde_json::from_str::<PendingDelivery>(&row.delivery) {
            Ok(delivery) => delivery,
            Err(error) => {
                record_failure(pool, &row, &format!("invalid queued delivery: {error}")).await?;
                continue;
            }
        };
        match sender::send(&delivery, client, allow_private_smtp_hosts).await {
            Ok(()) => store::delete(pool, row.id).await?,
            Err(error) => {
                tracing::warn!(
                    "{} notification delivery failed: {error}",
                    delivery.provider
                );
                record_failure(pool, &row, &error).await?;
            }
        }
    }
    Ok(())
}

async fn record_failure(pool: &SqlitePool, row: &OutboxRow, error: &str) -> sqlx::Result<()> {
    let exponent = u32::try_from(row.attempts.clamp(0, 7)).unwrap_or(7);
    let delay_seconds = INITIAL_RETRY_SECONDS
        .saturating_mul(2_u64.saturating_pow(exponent))
        .min(MAX_RETRY_SECONDS);
    store::record_failure(pool, row, error, delay_seconds).await
}
