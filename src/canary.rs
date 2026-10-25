use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;
use utoipa::ToSchema;

const MAX_ERROR_CHARS: usize = 500;
const HISTORY_RETENTION: &str = "-365 days";

#[derive(Clone, Debug)]
pub struct CanaryConfig {
    pub enabled: bool,
    pub url: Option<String>,
    pub timeout_secs: u64,
}

#[derive(Clone, Copy, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CanaryState {
    Disabled,
    Unknown,
    Healthy,
    Degraded,
}

#[derive(Debug, FromRow)]
struct CanarySettingsRow {
    enabled: bool,
    url: Option<String>,
    timeout_secs: i64,
    last_checked_at: Option<DateTime<Utc>>,
    last_succeeded_at: Option<DateTime<Utc>>,
    last_error: Option<String>,
    settings_revision: i64,
}

#[derive(Debug)]
pub struct CanarySettings {
    pub config: CanaryConfig,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_succeeded_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub settings_revision: i64,
}

impl CanarySettings {
    pub fn state(&self) -> CanaryState {
        if !self.config.enabled {
            CanaryState::Disabled
        } else if self.last_checked_at.is_none() {
            CanaryState::Unknown
        } else if self.last_error.is_some() {
            CanaryState::Degraded
        } else {
            CanaryState::Healthy
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
enum CanaryHealth {
    Healthy,
    Failing,
}

#[derive(Debug, FromRow)]
pub struct CanaryHistorySummary {
    pub active_since: Option<DateTime<Utc>>,
    pub last_ended_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanaryResultWrite {
    Recorded,
    DiscardedStaleRevision,
}

pub async fn load_settings(pool: &SqlitePool) -> Result<CanarySettings, sqlx::Error> {
    let row = sqlx::query_as::<_, CanarySettingsRow>(
        "SELECT enabled, url, timeout_secs,
                last_checked_at, last_succeeded_at, last_error, settings_revision
         FROM canary_settings WHERE id = 1",
    )
    .fetch_one(pool)
    .await?;
    let timeout_secs =
        u64::try_from(row.timeout_secs).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    Ok(CanarySettings {
        config: CanaryConfig {
            enabled: row.enabled,
            url: row.url,
            timeout_secs,
        },
        last_checked_at: row.last_checked_at,
        last_succeeded_at: row.last_succeeded_at,
        last_error: row.last_error,
        settings_revision: row.settings_revision,
    })
}

pub async fn update_settings(pool: &SqlitePool, config: &CanaryConfig) -> Result<(), sqlx::Error> {
    let timeout_secs =
        i64::try_from(config.timeout_secs).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    let mut transaction = pool.begin().await?;
    sqlx::query(
        "UPDATE canary_settings
         SET enabled = ?, timeout_secs = ?, url = ?,
             last_checked_at = CASE WHEN target_is_unchanged THEN last_checked_at ELSE NULL END,
             last_succeeded_at = CASE WHEN target_is_unchanged THEN last_succeeded_at ELSE NULL END,
             last_error = CASE WHEN target_is_unchanged THEN last_error ELSE NULL END,
             settings_revision = settings_revision + 1, updated_at = CURRENT_TIMESTAMP
         FROM (SELECT url IS ? AND enabled = ? AS target_is_unchanged
               FROM canary_settings WHERE id = 1) AS prior
         WHERE id = 1",
    )
    .bind(config.enabled)
    .bind(timeout_secs)
    .bind(&config.url)
    .bind(&config.url)
    .bind(config.enabled)
    .execute(&mut *transaction)
    .await?;
    sqlx::query(
        "UPDATE canary_history SET ended_at = CURRENT_TIMESTAMP
         WHERE ended_at IS NULL
           AND NOT EXISTS (SELECT 1 FROM canary_settings
                           WHERE id = 1 AND enabled = 1 AND url = canary_history.url)",
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await
}

pub async fn delete_old_history(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let deleted = sqlx::query(
        "DELETE FROM canary_history
         WHERE ended_at IS NOT NULL AND ended_at < datetime('now', ?)",
    )
    .bind(HISTORY_RETENTION)
    .execute(pool)
    .await?;
    Ok(deleted.rows_affected())
}

pub async fn load_history_summary(pool: &SqlitePool) -> Result<CanaryHistorySummary, sqlx::Error> {
    sqlx::query_as::<_, CanaryHistorySummary>(
        "SELECT (SELECT active_since FROM canary_history WHERE ended_at IS NULL) AS active_since,
                (SELECT MAX(ended_at) FROM canary_history) AS last_ended_at",
    )
    .fetch_one(pool)
    .await
}

async fn record_health_change(
    connection: &mut sqlx::SqliteConnection,
    health: CanaryHealth,
) -> Result<(), sqlx::Error> {
    let open_period: Option<(i64, CanaryHealth)> =
        sqlx::query_as("SELECT id, state FROM canary_history WHERE ended_at IS NULL")
            .fetch_optional(&mut *connection)
            .await?;
    if open_period.is_some_and(|(_, open_health)| open_health == health) {
        return Ok(());
    }
    let open_period_id = open_period.map(|(id, _)| id);
    sqlx::query("UPDATE canary_history SET ended_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(open_period_id)
        .execute(&mut *connection)
        .await?;
    sqlx::query(
        "INSERT INTO canary_history (state, url, active_since)
         SELECT ?1, url,
                COALESCE((SELECT active_since FROM canary_history WHERE id = ?2), CURRENT_TIMESTAMP)
         FROM canary_settings WHERE id = 1",
    )
    .bind(health)
    .bind(open_period_id)
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub async fn check(client: &Client, config: &CanaryConfig) -> Result<(), String> {
    let url = config
        .url
        .as_deref()
        .ok_or_else(|| "Canary URL is not configured".to_owned())?;
    client
        .head(url)
        .timeout(Duration::from_secs(config.timeout_secs))
        .send()
        .await
        .map(|_| ())
        .map_err(|error| bounded_error_message(&error.to_string()))
}

pub async fn record_result_if_current(
    pool: &SqlitePool,
    expected_settings_revision: i64,
    result: &Result<(), String>,
) -> Result<CanaryResultWrite, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let outcome = match result {
        Ok(()) => {
            sqlx::query(
                "UPDATE canary_settings
                 SET last_checked_at = CURRENT_TIMESTAMP,
                     last_succeeded_at = CURRENT_TIMESTAMP,
                     last_error = NULL
                 WHERE id = 1 AND settings_revision = ?",
            )
            .bind(expected_settings_revision)
            .execute(&mut *transaction)
            .await?
        }
        Err(error) => {
            sqlx::query(
                "UPDATE canary_settings
                 SET last_checked_at = CURRENT_TIMESTAMP, last_error = ?
                 WHERE id = 1 AND settings_revision = ?",
            )
            .bind(error)
            .bind(expected_settings_revision)
            .execute(&mut *transaction)
            .await?
        }
    };
    if outcome.rows_affected() == 0 {
        tracing::warn!("Discarding stale canary result: the settings changed mid-check");
        return Ok(CanaryResultWrite::DiscardedStaleRevision);
    }
    let health = if result.is_ok() {
        CanaryHealth::Healthy
    } else {
        CanaryHealth::Failing
    };
    record_health_change(&mut transaction, health).await?;
    transaction.commit().await?;
    Ok(CanaryResultWrite::Recorded)
}

fn bounded_error_message(message: &str) -> String {
    let mut chars = message.chars();
    let bounded: String = chars.by_ref().take(MAX_ERROR_CHARS).collect();
    if chars.next().is_some() {
        format!(
            "{}...",
            bounded
                .chars()
                .take(MAX_ERROR_CHARS - 3)
                .collect::<String>()
        )
    } else {
        bounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://canary.waghorn.tech/health";

    #[derive(Debug, FromRow)]
    struct HistoryRow {
        state: CanaryHealth,
        url: String,
        active_since: DateTime<Utc>,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
    }

    async fn backdate_active_since(pool: &SqlitePool) {
        sqlx::query("UPDATE canary_history SET active_since = '2026-01-01 00:00:00'")
            .execute(pool)
            .await
            .unwrap();
    }

    async fn history(pool: &SqlitePool) -> Vec<HistoryRow> {
        sqlx::query_as(
            "SELECT state, url, active_since, started_at, ended_at FROM canary_history ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn configure(pool: &SqlitePool, url: &str, enabled: bool, timeout_secs: u64) -> i64 {
        update_settings(
            pool,
            &CanaryConfig {
                enabled,
                url: Some(url.to_owned()),
                timeout_secs,
            },
        )
        .await
        .unwrap();
        load_settings(pool).await.unwrap().settings_revision
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_result_from_a_superseded_revision_is_discarded(pool: SqlitePool) {
        let stale_revision = configure(&pool, URL, true, 1).await;
        let current_revision = configure(&pool, URL, true, 30).await;
        assert_ne!(stale_revision, current_revision);
        record_result_if_current(&pool, stale_revision, &Err("boom".to_owned()))
            .await
            .unwrap();
        let settings = load_settings(&pool).await.unwrap();
        assert!(settings.last_checked_at.is_none());
        assert!(settings.last_error.is_none());
        record_result_if_current(&pool, current_revision, &Err("boom".to_owned()))
            .await
            .unwrap();
        let settings = load_settings(&pool).await.unwrap();
        assert_eq!(settings.last_error.as_deref(), Some("boom"));
        assert!(settings.last_checked_at.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_history_survives_a_timeout_change_but_not_a_url_change(pool: SqlitePool) {
        let settings_revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, settings_revision, &Ok(()))
            .await
            .unwrap();
        assert!(
            load_settings(&pool)
                .await
                .unwrap()
                .last_succeeded_at
                .is_some()
        );

        configure(&pool, URL, true, 30).await;
        let settings = load_settings(&pool).await.unwrap();
        assert_eq!(settings.config.timeout_secs, 30);
        assert!(
            settings.last_succeeded_at.is_some(),
            "a timeout change must not discard canary history"
        );

        configure(&pool, "https://replacement.waghorn.tech/health", true, 30).await;
        let settings = load_settings(&pool).await.unwrap();
        assert!(
            settings.last_succeeded_at.is_none(),
            "a URL change must discard history measured against the old URL"
        );
        assert!(settings.last_checked_at.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_first_result_opens_a_period_with_its_health(pool: SqlitePool) {
        let revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        let history = history(&pool).await;
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].state, CanaryHealth::Healthy);
        assert_eq!(history[0].url, URL);
        assert_eq!(history[0].active_since, history[0].started_at);
        assert!(history[0].ended_at.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_health_change_starts_a_new_period_with_the_same_active_since(pool: SqlitePool) {
        let revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        backdate_active_since(&pool).await;
        record_result_if_current(&pool, revision, &Err("boom".to_owned()))
            .await
            .unwrap();
        record_result_if_current(&pool, revision, &Err("boom again".to_owned()))
            .await
            .unwrap();
        let history = history(&pool).await;
        assert_eq!(history.len(), 2, "a repeated failure must not add a period");
        assert_eq!(history[0].state, CanaryHealth::Healthy);
        assert!(history[0].ended_at.is_some());
        assert_eq!(history[1].state, CanaryHealth::Failing);
        assert!(history[1].ended_at.is_none());
        assert_eq!(
            history[1].active_since.to_rfc3339(),
            "2026-01-01T00:00:00+00:00",
            "a health change must keep the original active since"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_disabling_closes_the_open_period(pool: SqlitePool) {
        let revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        configure(&pool, URL, false, 1).await;
        let history = history(&pool).await;
        assert_eq!(history.len(), 1);
        assert!(history[0].ended_at.is_some());
        let summary = load_history_summary(&pool).await.unwrap();
        assert!(summary.active_since.is_none());
        assert_eq!(summary.last_ended_at, history[0].ended_at);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_timeout_change_keeps_the_open_period(pool: SqlitePool) {
        let revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        configure(&pool, URL, true, 30).await;
        let history = history(&pool).await;
        assert_eq!(history.len(), 1);
        assert!(history[0].ended_at.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_url_change_closes_the_period_and_the_next_result_starts_afresh(pool: SqlitePool) {
        let revision = configure(&pool, URL, true, 1).await;
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        backdate_active_since(&pool).await;
        let replacement_url = "https://replacement.waghorn.tech/health";
        let revision = configure(&pool, replacement_url, true, 1).await;
        assert!(history(&pool).await[0].ended_at.is_some());
        record_result_if_current(&pool, revision, &Ok(()))
            .await
            .unwrap();
        let history = history(&pool).await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].url, replacement_url);
        assert_eq!(history[1].active_since, history[1].started_at);
        assert!(history[1].ended_at.is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_stale_result_records_no_history(pool: SqlitePool) {
        let stale_revision = configure(&pool, URL, true, 1).await;
        configure(&pool, URL, true, 30).await;
        record_result_if_current(&pool, stale_revision, &Ok(()))
            .await
            .unwrap();
        assert!(history(&pool).await.is_empty());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_old_history_is_deleted_but_recent_and_open_periods_stay(pool: SqlitePool) {
        sqlx::query(
            "INSERT INTO canary_history (state, url, active_since, started_at, ended_at) VALUES
             ('healthy', ?1, datetime('now', '-500 days'), datetime('now', '-500 days'), datetime('now', '-400 days')),
             ('failing', ?1, datetime('now', '-500 days'), datetime('now', '-400 days'), datetime('now', '-10 days')),
             ('healthy', ?1, datetime('now', '-500 days'), datetime('now', '-10 days'), NULL)",
        )
        .bind(URL)
        .execute(&pool)
        .await
        .unwrap();
        assert_eq!(delete_old_history(&pool).await.unwrap(), 1);
        let history = history(&pool).await;
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].state, CanaryHealth::Failing);
        assert!(history[1].ended_at.is_none());
    }
}
