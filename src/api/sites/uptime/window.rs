use super::timeline::{
    BlockedTimes, MeasuredUptime, OutageTimes, TimeRange, UptimeRecords, measure_uptime_in_range,
    monitor_running_ranges, seconds_in_state, site_blocked_ranges, total_seconds,
};
use crate::api::sites::canary_warning::normalised_url_host;
use crate::api::sites::responses::{BlockedSegment, UptimeDay, WatchStatus};
use crate::canary::CanaryPeriod;
use crate::monitoring::periods::MonitoringPeriod;
use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Span, Timestamp};

pub(in crate::api::sites) struct UptimeWindow<'a> {
    pub site_url: &'a str,
    pub monitored_from: Timestamp,
    pub now: Timestamp,
    pub time_zone: TimeZone,
    pub days: u64,
    pub outages: &'a [OutageTimes],
    pub blocked_periods: &'a [BlockedTimes],
    pub canary_periods: &'a [CanaryPeriod],
    pub monitoring_periods: &'a [MonitoringPeriod],
}

pub(in crate::api::sites) struct WindowSummary {
    pub uptime_basis_points: Option<i64>,
    pub unwatched_seconds: i64,
}

pub(in crate::api::sites) fn uptime_window_start(
    now: Timestamp,
    time_zone: &TimeZone,
    days: u64,
) -> Timestamp {
    let today = local_date(now, time_zone);
    let first_day = days_before(today, days.saturating_sub(1)).unwrap_or(today);
    start_of_local_day(first_day, time_zone).unwrap_or(now)
}

pub(in crate::api::sites) fn daily_uptime(window: &UptimeWindow<'_>) -> Vec<UptimeDay> {
    let today = local_date(window.now, &window.time_zone);
    let running_ranges = monitor_running_ranges(window.monitoring_periods, window.now);
    let blocked_ranges = site_blocked_ranges(window.blocked_periods, window.now);
    let records = UptimeRecords {
        running_ranges: &running_ranges,
        blocked_ranges: &blocked_ranges,
        canary_periods: window.canary_periods,
        outages: window.outages,
        now: window.now,
    };
    let canary_shares_site_host = |canary_url: &str| {
        normalised_url_host(canary_url).is_some_and(|canary_host| {
            normalised_url_host(window.site_url).is_some_and(|site_host| site_host == canary_host)
        })
    };
    (0..window.days)
        .rev()
        .filter_map(|days_ago| days_before(today, days_ago))
        .filter_map(|date| {
            let day_start = start_of_local_day(date, &window.time_zone)?;
            let day_end = date
                .tomorrow()
                .ok()
                .and_then(|next| start_of_local_day(next, &window.time_zone))
                .unwrap_or(window.now);
            let day_range = TimeRange {
                start: day_start.max(window.monitored_from),
                end: day_end.min(window.now),
            };
            if day_range.duration_seconds() == 0 {
                return Some(day_before_monitoring_started(date));
            }
            let outage_count = window
                .outages
                .iter()
                .filter(|outage| {
                    let outage_end = outage
                        .ended_at
                        .map_or(window.now, jiff_sqlx::Timestamp::to_jiff);
                    day_range.overlap_seconds(outage.started_at.to_jiff(), outage_end) > 0
                })
                .count();
            let MeasuredUptime {
                timeline,
                blocked,
                watched,
            } = measure_uptime_in_range(&day_range, &records, canary_shares_site_host);
            Some(UptimeDay {
                date,
                monitoring_started: true,
                uptime_basis_points: watched.uptime_basis_points(),
                watched_seconds: watched.watched_seconds,
                down_seconds: watched.down_seconds,
                outage_count: i64::try_from(outage_count).unwrap_or(i64::MAX),
                canary_healthy_seconds: seconds_in_state(&timeline, WatchStatus::CanaryHealthy),
                canary_shares_host_seconds: seconds_in_state(
                    &timeline,
                    WatchStatus::CanarySharesHost,
                ),
                canary_failing_seconds: seconds_in_state(&timeline, WatchStatus::CanaryFailing),
                no_canary_seconds: seconds_in_state(&timeline, WatchStatus::NoCanary),
                not_running_seconds: seconds_in_state(&timeline, WatchStatus::NotRunning),
                blocked_seconds: total_seconds(&blocked),
                watch_segments: timeline,
                blocked_segments: blocked
                    .iter()
                    .map(|range| BlockedSegment {
                        started_at: range.start,
                        ended_at: range.end,
                    })
                    .collect(),
            })
        })
        .collect()
}

pub(in crate::api::sites) fn window_summary(window: &UptimeWindow<'_>) -> WindowSummary {
    let window_range = TimeRange {
        start: uptime_window_start(window.now, &window.time_zone, window.days)
            .max(window.monitored_from),
        end: window.now,
    };
    let running_ranges = monitor_running_ranges(window.monitoring_periods, window.now);
    let blocked_ranges = site_blocked_ranges(window.blocked_periods, window.now);
    let records = UptimeRecords {
        running_ranges: &running_ranges,
        blocked_ranges: &blocked_ranges,
        canary_periods: window.canary_periods,
        outages: window.outages,
        now: window.now,
    };
    let watched = measure_uptime_in_range(&window_range, &records, |_| false).watched;
    WindowSummary {
        uptime_basis_points: watched.uptime_basis_points(),
        unwatched_seconds: window_range
            .duration_seconds()
            .saturating_sub(watched.watched_seconds)
            .max(0),
    }
}

fn local_date(instant: Timestamp, time_zone: &TimeZone) -> Date {
    instant.to_zoned_datetime(time_zone.clone()).date()
}

fn days_before(date: Date, days: u64) -> Option<Date> {
    let days = i64::try_from(days).ok()?;
    date.checked_sub(Span::new().try_days(days).ok()?).ok()
}

fn start_of_local_day(date: Date, time_zone: &TimeZone) -> Option<Timestamp> {
    let start = date
        .to_zoned_datetime(time_zone.clone())
        .ok()?
        .start_of_day()
        .ok()?;
    Some(start.timestamp())
}

fn day_before_monitoring_started(date: Date) -> UptimeDay {
    UptimeDay {
        date,
        monitoring_started: false,
        uptime_basis_points: None,
        watched_seconds: 0,
        down_seconds: 0,
        outage_count: 0,
        canary_healthy_seconds: 0,
        canary_shares_host_seconds: 0,
        canary_failing_seconds: 0,
        no_canary_seconds: 0,
        not_running_seconds: 0,
        blocked_seconds: 0,
        blocked_segments: Vec::new(),
        watch_segments: Vec::new(),
    }
}
