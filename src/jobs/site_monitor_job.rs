use crate::config::{CANARY_TIMEOUT_SECS, CANARY_URL};
use crate::models::SiteRow;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::{Client, StatusCode};
use sqlx::SqlitePool;
use std::time::Duration;

pub struct CheckExpectation {
    pub expected_status: u16,
    pub expected_text: Option<String>,
}

pub struct ProbeResult {
    pub is_up: bool,
    pub status_code: Option<StatusCode>,
    pub latency_ms: Option<u128>,
    pub expected_response: bool,
    pub error_message: Option<String>,
}

pub const MAX_CONCURRENT_CHECKS: usize = 10;

pub async fn check_all_sites(client: &Client, pool: &SqlitePool) {
    if client
        .head(CANARY_URL)
        .timeout(Duration::from_secs(CANARY_TIMEOUT_SECS))
        .send()
        .await
        .is_err()
    {
        tracing::warn!("Canary check failed. Skipping sites check. Network issue?");
        return;
    }
    tracing::info!("Checking all sites");
    let sites = sqlx::query_as::<_, SiteRow>(
        "SELECT id, name, url, expected_status, expected_text, is_up FROM sites",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    if sites.is_empty() {
        tracing::info!("No sites found");
        return;
    }
    let site_count = sites.len();
    stream::iter(sites)
        .map(|site| check_single_site(client, pool, site))
        .buffer_unordered(MAX_CONCURRENT_CHECKS)
        .collect::<Vec<()>>()
        .await;
    tracing::info!("Finished checking {} sites", site_count);
}

async fn check_single_site(client: &Client, pool: &SqlitePool, site: SiteRow) {
    tracing::info!("Checking site {}", site.name);
    let check = CheckExpectation {
        expected_status: u16::try_from(site.expected_status).unwrap_or(200),
        expected_text: site.expected_text.clone(),
    };
    let probe_result = probe_site(client, &site.url, &check).await;
    update_site_status(pool, &site, &probe_result).await;
}

pub async fn probe_site(client: &Client, url: &str, check: &CheckExpectation) -> ProbeResult {
    let start = std::time::Instant::now();
    match client
        .get(url)
        .timeout(Duration::from_secs(20))
        .send()
        .await
    {
        Ok(res) => {
            let status = res.status();
            let latency_ms = start.elapsed().as_millis();
            let status_matches = status.as_u16() == check.expected_status;
            let text_matches = if let Some(expected_text) = &check.expected_text {
                match res.text().await {
                    Ok(body) => body.contains(expected_text),
                    Err(_) => false,
                }
            } else {
                true
            };
            let expected_response = status_matches && text_matches;
            ProbeResult {
                is_up: status.is_success(),
                status_code: Some(status),
                latency_ms: Some(latency_ms),
                expected_response,
                error_message: None,
            }
        }
        Err(e) => ProbeResult {
            is_up: false,
            status_code: None,
            latency_ms: None,
            expected_response: false,
            error_message: Some(e.to_string().chars().take(500).collect()),
        },
    }
}

pub async fn update_site_status(pool: &SqlitePool, site: &SiteRow, status: &ProbeResult) {
    sqlx::query(
        "UPDATE sites SET is_up = ?, last_checked_at = ?, last_response_time_ms = ?, expected_response = ? WHERE id = ?"
    )
        .bind(i64::from(status.is_up))
        .bind(Utc::now())
        .bind(status.latency_ms.map(|ms| i64::try_from(ms).unwrap_or(i64::MAX)))
        .bind(i64::from(status.expected_response))
        .bind(site.id)
        .execute(pool)
        .await
        .map_err(|e| tracing::error!("Failed to update site status for site {}: {}", site.id, e))
        .ok();
    if site.is_up == 1 && !status.is_up {
        sqlx::query("INSERT INTO outages (site_id, http_status, error_message) VALUES (?, ?, ?)")
            .bind(site.id)
            .bind(status.status_code.map(|c| i64::from(c.as_u16())))
            .bind(&status.error_message)
            .execute(pool)
            .await
            .map_err(|e| tracing::error!("Failed to insert outage for site {}: {}", site.id, e))
            .ok();
    }
    if site.is_up == 0 && status.is_up {
        sqlx::query("UPDATE outages SET ended_at = ? WHERE site_id = ? AND ended_at IS NULL")
            .bind(Utc::now())
            .bind(site.id)
            .execute(pool)
            .await
            .map_err(|e| tracing::error!("Failed to close outage for site {}: {}", site.id, e))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::insert_test_site;
    use sqlx::SqlitePool;

    fn mock_site_down_result() -> ProbeResult {
        ProbeResult {
            is_up: false,
            status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            latency_ms: Some(500),
            expected_response: false,
            error_message: Some(String::from("Server is cooked")),
        }
    }

    fn mock_site_up_result() -> ProbeResult {
        ProbeResult {
            is_up: true,
            status_code: Some(StatusCode::OK),
            latency_ms: Some(100),
            expected_response: true,
            error_message: None,
        }
    }
    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_created_when_site_is_down(pool: sqlx::SqlitePool) {
        let inserted_available_site = insert_test_site(&pool, 1).await;
        let mock_down_result = mock_site_down_result();
        update_site_status(&pool, &inserted_available_site, &mock_down_result).await;
        let count: (i64,) = sqlx::query_as("SELECT  COUNT(*) FROM outages WHERE site_id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_outage_closed_when_site_recovers(pool: sqlx::SqlitePool) {
        let inserted_down_site = insert_test_site(&pool, 0).await;
        sqlx::query("INSERT INTO outages (site_id, http_status, error_message) VALUES (?, ?, ?)")
            .bind(inserted_down_site.id)
            .bind(500)
            .bind(String::from("Server cooked"))
            .execute(&pool)
            .await
            .unwrap();
        update_site_status(&pool, &inserted_down_site, &mock_site_up_result()).await;
        let outage_ended: Option<String> =
            sqlx::query_scalar("SELECT ended_at FROM outages WHERE site_id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(outage_ended.is_some());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn test_no_duplicate_outage_when_already_down(pool: SqlitePool) {
        let inserted_down_site = insert_test_site(&pool, 0).await; // already down
        update_site_status(&pool, &inserted_down_site, &mock_site_down_result()).await;
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM outages")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}
