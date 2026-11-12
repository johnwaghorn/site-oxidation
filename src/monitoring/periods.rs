use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

pub const MAX_HEARTBEAT_GAP_SECONDS: i64 = 120;
const PERIOD_RETENTION: &str = "-365 days";

#[derive(Debug, FromRow)]
pub struct MonitoringPeriod {
    pub started_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

pub async fn record_heartbeat(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let extended = sqlx::query(
        "UPDATE monitoring_periods SET last_seen_at = CURRENT_TIMESTAMP
         WHERE id = (SELECT MAX(id) FROM monitoring_periods) AND last_seen_at >= datetime('now', ?)",
    )
    .bind(format!("-{MAX_HEARTBEAT_GAP_SECONDS} seconds"))
    .execute(pool)
    .await?;
    if extended.rows_affected() == 0 {
        sqlx::query("INSERT INTO monitoring_periods DEFAULT VALUES")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn delete_old_periods(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let deleted =
        sqlx::query("DELETE FROM monitoring_periods WHERE last_seen_at < datetime('now', ?)")
            .bind(PERIOD_RETENTION)
            .execute(pool)
            .await?;
    Ok(deleted.rows_affected())
}

pub async fn load_periods_in_uptime_calculation_window(
    pool: &SqlitePool,
    window_start: DateTime<Utc>,
) -> Result<Vec<MonitoringPeriod>, sqlx::Error> {
    sqlx::query_as::<_, MonitoringPeriod>(
        "SELECT started_at, last_seen_at FROM monitoring_periods
         WHERE last_seen_at > ?
         ORDER BY started_at",
    )
    .bind(window_start.format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn periods(pool: &SqlitePool) -> Vec<(String, String)> {
        sqlx::query_as("SELECT started_at, last_seen_at FROM monitoring_periods ORDER BY id")
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_first_heartbeat_starts_a_period(pool: SqlitePool) {
        record_heartbeat(&pool).await.unwrap();
        assert_eq!(periods(&pool).await.len(), 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_heartbeat_soon_after_the_last_extends_the_period(pool: SqlitePool) {
        sqlx::query(
            "INSERT INTO monitoring_periods (started_at, last_seen_at)
             VALUES (datetime('now', '-1 hour'), datetime('now', '-30 seconds'))",
        )
        .execute(&pool)
        .await
        .unwrap();
        record_heartbeat(&pool).await.unwrap();
        let periods = periods(&pool).await;
        assert_eq!(periods.len(), 1);
        assert!(periods[0].1 > periods[0].0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_heartbeat_after_a_long_gap_starts_a_new_period(pool: SqlitePool) {
        sqlx::query(
            "INSERT INTO monitoring_periods (started_at, last_seen_at)
             VALUES (datetime('now', '-1 hour'), datetime('now', '-10 minutes'))",
        )
        .execute(&pool)
        .await
        .unwrap();
        record_heartbeat(&pool).await.unwrap();
        assert_eq!(periods(&pool).await.len(), 2, "the gap stays unwatched");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_only_periods_that_ended_over_a_year_ago_are_deleted(pool: SqlitePool) {
        sqlx::query(
            "INSERT INTO monitoring_periods (started_at, last_seen_at) VALUES
             (datetime('now', '-500 days'), datetime('now', '-400 days')),
             (datetime('now', '-390 days'), datetime('now', '-300 days')),
             (datetime('now', '-1 hour'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(delete_old_periods(&pool).await.unwrap(), 1);
        assert_eq!(periods(&pool).await.len(), 2);
    }
}
