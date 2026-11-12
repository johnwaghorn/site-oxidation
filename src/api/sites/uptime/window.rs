use super::timeline::{
    OutageTimes, TimeRange, build_watch_timeline, measure_watched_time, monitor_running_ranges,
    seconds_in_state,
};
use crate::api::sites::canary_warning::normalised_url_host;
use crate::api::sites::responses::{UptimeDay, WatchStatus};
use crate::canary::CanaryPeriod;
use crate::monitoring::periods::MonitoringPeriod;
use chrono::{DateTime, Datelike, Days, NaiveDate, NaiveTime, Utc};
use jiff::Timestamp;
use jiff::civil::Date;
use jiff::tz::TimeZone;

pub(in crate::api::sites) struct UptimeWindow<'a> {
    pub site_url: &'a str,
    pub monitored_from: DateTime<Utc>,
    pub now: DateTime<Utc>,
    pub time_zone: TimeZone,
    pub days: u64,
    pub outages: &'a [OutageTimes],
    pub canary_periods: &'a [CanaryPeriod],
    pub monitoring_periods: &'a [MonitoringPeriod],
}

pub(in crate::api::sites) struct WindowSummary {
    pub uptime_basis_points: Option<i64>,
    pub unwatched_seconds: i64,
}

pub(in crate::api::sites) fn uptime_window_start(
    now: DateTime<Utc>,
    time_zone: &TimeZone,
    days: u64,
) -> DateTime<Utc> {
    let today = local_date(now, time_zone);
    let first_day = today
        .checked_sub_days(Days::new(days.saturating_sub(1)))
        .unwrap_or(today);
    start_of_local_day(first_day, time_zone)
}

pub(in crate::api::sites) fn daily_uptime(window: &UptimeWindow<'_>) -> Vec<UptimeDay> {
    let today = local_date(window.now, &window.time_zone);
    let running_ranges = monitor_running_ranges(window.monitoring_periods, window.now);
    let canary_shares_site_host = |canary_url: &str| {
        normalised_url_host(canary_url).is_some_and(|canary_host| {
            normalised_url_host(window.site_url).is_some_and(|site_host| site_host == canary_host)
        })
    };
    (0..window.days)
        .rev()
        .filter_map(|days_ago| today.checked_sub_days(Days::new(days_ago)))
        .map(|date| {
            let day_start = start_of_local_day(date, &window.time_zone);
            let day_end = date.succ_opt().map_or(window.now, |next| {
                start_of_local_day(next, &window.time_zone)
            });
            let day_range = TimeRange {
                start: day_start.max(window.monitored_from),
                end: day_end.min(window.now),
            };
            if day_range.duration_seconds() == 0 {
                return day_before_monitoring_started(date);
            }
            let outage_count = window
                .outages
                .iter()
                .filter(|outage| {
                    day_range
                        .overlap_seconds(outage.started_at, outage.ended_at.unwrap_or(window.now))
                        > 0
                })
                .count();
            let timeline = build_watch_timeline(
                &day_range,
                &running_ranges,
                window.canary_periods,
                window.now,
                canary_shares_site_host,
            );
            let watched = measure_watched_time(&timeline, window.outages, window.now);
            UptimeDay {
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
                watch_segments: timeline,
            }
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
    let timeline = build_watch_timeline(
        &window_range,
        &running_ranges,
        window.canary_periods,
        window.now,
        |_| false,
    );
    let watched = measure_watched_time(&timeline, window.outages, window.now);
    WindowSummary {
        uptime_basis_points: watched.uptime_basis_points(),
        unwatched_seconds: window_range
            .duration_seconds()
            .saturating_sub(watched.watched_seconds)
            .max(0),
    }
}

fn local_date(instant: DateTime<Utc>, time_zone: &TimeZone) -> NaiveDate {
    Timestamp::from_second(instant.timestamp())
        .ok()
        .and_then(|timestamp| {
            let date = timestamp.to_zoned(time_zone.clone()).date();
            NaiveDate::from_ymd_opt(
                date.year().into(),
                date.month().unsigned_abs().into(),
                date.day().unsigned_abs().into(),
            )
        })
        .unwrap_or_else(|| instant.date_naive())
}

/// Where the clocks skip midnight, the day starts at the first local time that exists.
fn start_of_local_day(date: NaiveDate, time_zone: &TimeZone) -> DateTime<Utc> {
    let first_instant = || {
        let year = i16::try_from(date.year()).ok()?;
        let month = i8::try_from(date.month()).ok()?;
        let day = i8::try_from(date.day()).ok()?;
        let start = Date::new(year, month, day)
            .ok()?
            .to_zoned(time_zone.clone())
            .ok()?
            .start_of_day()
            .ok()?;
        DateTime::from_timestamp(start.timestamp().as_second(), 0)
    };
    first_instant().unwrap_or_else(|| date.and_time(NaiveTime::MIN).and_utc())
}

fn day_before_monitoring_started(date: NaiveDate) -> UptimeDay {
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
        watch_segments: Vec::new(),
    }
}
