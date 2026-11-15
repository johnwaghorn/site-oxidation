use super::timeline::{
    BlockedTimes, OutageTimes, TimeRange, UptimeRecords, measure_uptime_in_range,
    monitor_running_ranges, site_blocked_ranges,
};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::api::sites::queries::{
    LIST_BLOCKED_PERIODS_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS,
    LIST_OUTAGES_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS,
};
use crate::api::sites::responses::SiteResponse;
use crate::canary;
use crate::monitoring::periods;
use chrono::{Days, Utc};
use sqlx::{FromRow, SqlitePool};
use std::collections::HashMap;

const SITE_UPTIME_WINDOW_DAYS: u64 = 90;

#[derive(FromRow)]
struct SiteOutageTimes {
    site_id: i64,
    #[sqlx(flatten)]
    times: OutageTimes,
}

#[derive(FromRow)]
struct SiteBlockedTimes {
    site_id: i64,
    #[sqlx(flatten)]
    times: BlockedTimes,
}

pub(crate) async fn set_site_uptime_figures(
    pool: &SqlitePool,
    sites: &mut [SiteResponse],
) -> Result<(), ApiErrorResponse> {
    for site in sites.iter_mut() {
        site.uptime_window_days = SITE_UPTIME_WINDOW_DAYS;
    }
    let Some(earliest_created_at) = sites
        .iter()
        .filter(|site| site.last_checked_at.is_some())
        .map(|site| site.created_at)
        .min()
    else {
        return Ok(());
    };
    let now = Utc::now();
    let window_start = now
        .checked_sub_days(Days::new(SITE_UPTIME_WINDOW_DAYS))
        .unwrap_or(now);
    let earliest_needed = earliest_created_at.max(window_start);
    let site_ids: Vec<i64> = sites.iter().map(|site| site.id).collect();
    let site_ids_json = serde_json::to_string(&site_ids)
        .map_err(|error| internal_err("Failed to encode site ids", error))?;
    let mut outages_by_site: HashMap<i64, Vec<OutageTimes>> = HashMap::new();
    for outage in
        sqlx::query_as::<_, SiteOutageTimes>(LIST_OUTAGES_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS)
            .bind(&site_ids_json)
            .bind(earliest_needed.format("%Y-%m-%d %H:%M:%S").to_string())
            .fetch_all(pool)
            .await
            .map_err(|error| internal_err("Failed to load site outages", error))?
    {
        outages_by_site
            .entry(outage.site_id)
            .or_default()
            .push(outage.times);
    }
    let mut blocked_periods_by_site: HashMap<i64, Vec<BlockedTimes>> = HashMap::new();
    for blocked in sqlx::query_as::<_, SiteBlockedTimes>(
        LIST_BLOCKED_PERIODS_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS,
    )
    .bind(&site_ids_json)
    .bind(earliest_needed.format("%Y-%m-%d %H:%M:%S").to_string())
    .fetch_all(pool)
    .await
    .map_err(|error| internal_err("Failed to load site blocked periods", error))?
    {
        blocked_periods_by_site
            .entry(blocked.site_id)
            .or_default()
            .push(blocked.times);
    }
    let monitoring_periods =
        periods::load_periods_in_uptime_calculation_window(pool, earliest_needed)
            .await
            .map_err(|error| internal_err("Failed to load monitoring periods", error))?;
    let canary_periods = canary::load_periods_in_uptime_calculation_window(pool, earliest_needed)
        .await
        .map_err(|error| internal_err("Failed to load canary history", error))?;
    let running_ranges = monitor_running_ranges(&monitoring_periods, now);
    for site in sites {
        if site.last_checked_at.is_none() {
            continue;
        }
        let window_range = TimeRange {
            start: site.created_at.max(window_start),
            end: now,
        };
        let blocked_periods = blocked_periods_by_site
            .get(&site.id)
            .map_or(&[][..], Vec::as_slice);
        let records = UptimeRecords {
            running_ranges: &running_ranges,
            blocked_ranges: &site_blocked_ranges(blocked_periods, now),
            canary_periods: &canary_periods,
            outages: outages_by_site.get(&site.id).map_or(&[][..], Vec::as_slice),
            now,
        };
        site.uptime_basis_points = measure_uptime_in_range(&window_range, &records, |_| false)
            .watched
            .uptime_basis_points();
    }
    Ok(())
}
