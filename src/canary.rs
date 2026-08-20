use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;
use utoipa::ToSchema;

const MAX_ERROR_CHARS: usize = 500;

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
    .execute(pool)
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
            .execute(pool)
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
            .execute(pool)
            .await?
        }
    };
    if outcome.rows_affected() == 0 {
        tracing::warn!("Discarding stale canary result: the settings changed mid-check");
        return Ok(CanaryResultWrite::DiscardedStaleRevision);
    }
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
}
