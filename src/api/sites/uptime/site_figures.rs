use super::timeline::{
    OutageTimes, TimeRange, build_watch_timeline, measure_watched_time, monitor_running_ranges,
};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::api::sites::queries::LIST_OUTAGES_IN_UPTIME_CALCULATION_WINDOW_FOR_SITE_IDS;
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
            .bind(site_ids_json)
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
        let timeline =
            build_watch_timeline(&window_range, &running_ranges, &canary_periods, now, |_| {
                false
            });
        let outages = outages_by_site.get(&site.id).map_or(&[][..], Vec::as_slice);
        site.uptime_basis_points =
            measure_watched_time(&timeline, outages, now).uptime_basis_points();
    }
    Ok(())
}
