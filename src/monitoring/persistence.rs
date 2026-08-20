use crate::models::site::{CertStatus, SiteRow};
use crate::notifications::{Notifier, planning};
use crate::probe::cert::CertCheck;
use crate::probe::http::{ProbeResult, UNKNOWN_PROBE_ERROR_MESSAGE};
use chrono::Utc;
use sqlx::{Sqlite, SqlitePool, Transaction};

pub(super) enum SiteTransition {
    WentDown,
    Recovered,
    BecameBlocked,
    NoChange,
}

pub(super) async fn persist_site_statuses(
    pool: &SqlitePool,
    sites: &[SiteRow],
    result: &ProbeResult,
    notifier: &Notifier,
) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;
    let mut went_down = Vec::new();
    let mut recovered = Vec::new();
    let mut blocked = Vec::new();
    for site in sites {
        match update_site_status(&mut transaction, site, result).await? {
            SiteTransition::WentDown => went_down.push(site),
            SiteTransition::Recovered => recovered.push(site),
            SiteTransition::BecameBlocked => blocked.push(site),
            SiteTransition::NoChange => {}
        }
    }
    let mut deliveries = planning::site_down(&went_down, result)?;
    deliveries.extend(planning::site_recovered(&recovered)?);
    notifier.enqueue(&mut transaction, &deliveries).await?;
    transaction.commit().await?;
    for site in went_down {
        tracing::warn!(
            "Site '{}' is DOWN (status: {}) - {}",
            site.name,
            result
                .status_code
                .map_or_else(|| "N/A".to_owned(), |code| code.to_string()),
            result
                .error_message
                .as_deref()
                .unwrap_or(UNKNOWN_PROBE_ERROR_MESSAGE)
        );
    }
    for site in recovered {
        tracing::info!("Site '{}' is back UP", site.name);
    }
    for site in blocked {
        tracing::warn!(
            "Site '{}' probe is BLOCKED - {}",
            site.name,
            result
                .error_message
                .as_deref()
                .unwrap_or(UNKNOWN_PROBE_ERROR_MESSAGE)
        );
    }
    Ok(())
}

pub(super) async fn persist_site_cert_results(
    pool: &SqlitePool,
    sites: &[SiteRow],
    cert: &CertCheck,
    notifier: &Notifier,
) -> anyhow::Result<()> {
    let mut transaction = pool.begin().await?;
    let newly_expiring: Vec<&SiteRow> = sites
        .iter()
        .filter(|site| cert_newly_expiring(site, cert))
        .collect();
    for site in sites {
        update_site_cert_status(&mut transaction, site.id, cert).await?;
    }
    let deliveries = planning::cert_expiring(&newly_expiring, cert)?;
    notifier.enqueue(&mut transaction, &deliveries).await?;
    transaction.commit().await?;
    Ok(())
}

fn cert_newly_expiring(site: &SiteRow, cert: &CertCheck) -> bool {
    matches!(
        cert.status,
        CertStatus::Expiring | CertStatus::Critical | CertStatus::Expired
    ) && site.cert_status != Some(cert.status)
}

pub(super) async fn update_site_status(
    transaction: &mut Transaction<'_, Sqlite>,
    site: &SiteRow,
    result: &ProbeResult,
) -> sqlx::Result<SiteTransition> {
    sqlx::query(
        "UPDATE sites SET status = ?, last_checked_at = ?, last_response_time_ms = ? WHERE id = ?",
    )
    .bind(result.status)
    .bind(Utc::now())
    .bind(
        result
            .latency_ms
            .map(|ms| i64::try_from(ms).unwrap_or(i64::MAX)),
    )
    .bind(site.id)
    .execute(&mut **transaction)
    .await?;
    if !site.status.is_down() && result.status.is_down() {
        sqlx::query(
            "INSERT INTO outages (site_id, http_status, error_message, expected_status) VALUES (?, ?, ?, ?)",
        )
        .bind(site.id)
        .bind(result.status_code.map(|c| i64::from(c.as_u16())))
        .bind(&result.error_message)
        .bind(site.expected_status)
        .execute(&mut **transaction)
        .await?;
        return Ok(SiteTransition::WentDown);
    }
    if site.status.is_down() && !result.status.is_down() {
        sqlx::query("UPDATE outages SET ended_at = ? WHERE site_id = ? AND ended_at IS NULL")
            .bind(Utc::now())
            .bind(site.id)
            .execute(&mut **transaction)
            .await?;
        if result.status.is_up() {
            return Ok(SiteTransition::Recovered);
        }
    }
    if !site.status.is_blocked() && result.status.is_blocked() {
        return Ok(SiteTransition::BecameBlocked);
    }
    Ok(SiteTransition::NoChange)
}

async fn update_site_cert_status(
    transaction: &mut Transaction<'_, Sqlite>,
    site_id: i64,
    cert: &CertCheck,
) -> sqlx::Result<()> {
    sqlx::query(
        "UPDATE sites SET cert_status = ?, cert_expires_at = ?, cert_checked_at = ? WHERE id = ?",
    )
    .bind(cert.status)
    .bind(cert.expires_at)
    .bind(Utc::now())
    .bind(site_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub(super) async fn clear_site_cert(pool: &SqlitePool, site_id: i64) {
    sqlx::query(
        "UPDATE sites SET cert_status = NULL, cert_expires_at = NULL, cert_checked_at = NULL WHERE id = ?",
    )
    .bind(site_id)
    .execute(pool)
    .await
    .map_err(|e| tracing::error!("Failed to clear cert status for site {}: {}", site_id, e))
    .ok();
}
