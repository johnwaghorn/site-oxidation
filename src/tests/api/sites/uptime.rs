use super::{OutageTimes, UptimeWindow, daily_uptime, window_summary};
use crate::api::sites::responses::{UptimeDay, WatchStatus};
use crate::canary::{CanaryHealth, CanaryPeriod};
use crate::monitoring::periods::MonitoringPeriod;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use std::sync::LazyLock;

const SITE_URL: &str = "https://waghorn.tech";

static ALWAYS_RUNNING: LazyLock<[MonitoringPeriod; 1]> =
    LazyLock::new(|| [period("2026-01-01 00:00:00", "2027-01-01 00:00:00")]);

fn utc(timestamp: &str) -> DateTime<Utc> {
    chrono::NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S")
        .unwrap()
        .and_utc()
}

fn outage(started_at: &str, ended_at: Option<&str>) -> OutageTimes {
    OutageTimes {
        started_at: utc(started_at),
        ended_at: ended_at.map(utc),
    }
}

fn canary(
    state: CanaryHealth,
    url: &str,
    started_at: &str,
    ended_at: Option<&str>,
) -> CanaryPeriod {
    CanaryPeriod {
        state,
        url: url.to_owned(),
        started_at: utc(started_at),
        ended_at: ended_at.map(utc),
    }
}

fn period(started_at: &str, last_seen_at: &str) -> MonitoringPeriod {
    MonitoringPeriod {
        started_at: utc(started_at),
        last_seen_at: utc(last_seen_at),
    }
}

fn window<'a>(
    now: &str,
    time_zone: Tz,
    outages: &'a [OutageTimes],
    canary_periods: &'a [CanaryPeriod],
) -> UptimeWindow<'a> {
    UptimeWindow {
        site_url: SITE_URL,
        monitored_from: utc("2026-01-01 00:00:00"),
        now: utc(now),
        time_zone,
        days: 7,
        outages,
        canary_periods,
        monitoring_periods: &*ALWAYS_RUNNING,
    }
}

fn segment_times(day: &UptimeDay) -> Vec<(WatchStatus, String, String)> {
    day.watch_segments
        .iter()
        .map(|segment| {
            (
                segment.state,
                segment.started_at.format("%H:%M").to_string(),
                segment.ended_at.format("%H:%M").to_string(),
            )
        })
        .collect()
}

fn expected_segments(segments: &[(WatchStatus, &str, &str)]) -> Vec<(WatchStatus, String, String)> {
    segments
        .iter()
        .map(|(state, start, end)| (*state, (*start).to_owned(), (*end).to_owned()))
        .collect()
}

fn day<'a>(days: &'a [UptimeDay], date: &str) -> &'a UptimeDay {
    days.iter()
        .find(|day| day.date.to_string() == date)
        .unwrap_or_else(|| panic!("no entry for {date}"))
}

#[test]
fn test_outage_spanning_midnight_is_split_across_days() {
    let outages = [outage("2026-10-17 23:30:00", Some("2026-10-18 00:30:00"))];
    let days = daily_uptime(&window("2026-10-20 12:00:00", Tz::UTC, &outages, &[]));
    for date in ["2026-10-17", "2026-10-18"] {
        assert_eq!(day(&days, date).down_seconds, 1800, "{date}");
        assert_eq!(day(&days, date).outage_count, 1, "{date}");
    }
}

#[test]
fn test_days_start_at_local_midnight_in_summer_time() {
    // 23:30 UTC on 8 July is 00:30 on 9 July in London.
    let outages = [outage("2026-07-08 23:30:00", Some("2026-07-08 23:45:00"))];
    let days = daily_uptime(&window(
        "2026-07-10 12:00:00",
        Tz::Europe__London,
        &outages,
        &[],
    ));
    assert_eq!(day(&days, "2026-07-08").down_seconds, 0);
    assert_eq!(day(&days, "2026-07-09").down_seconds, 900);
}

#[test]
fn test_clock_change_day_is_25_hours_long() {
    // 25 Oct 2026 in London runs from 23:00 UTC on the 24th to 00:00 UTC on the 26th.
    let outages = [outage("2026-10-25 10:00:00", Some("2026-10-25 10:15:00"))];
    let days = daily_uptime(&window(
        "2026-10-26 12:00:00",
        Tz::Europe__London,
        &outages,
        &[],
    ));
    assert_eq!(
        day(&days, "2026-10-25").uptime_basis_points,
        Some((90_000 - 900) * 10_000 / 90_000)
    );
}

#[test]
fn test_day_starts_after_the_gap_when_the_clocks_skip_midnight() {
    // In Santiago the clocks jump from 00:00 to 01:00 on 6 Sep 2026, so that day has 23 hours.
    let days = daily_uptime(&window(
        "2026-09-08 12:00:00",
        Tz::America__Santiago,
        &[],
        &[],
    ));
    assert_eq!(day(&days, "2026-09-05").watched_seconds, 24 * 3600);
    assert_eq!(day(&days, "2026-09-06").watched_seconds, 23 * 3600);
    assert_eq!(day(&days, "2026-09-07").watched_seconds, 24 * 3600);
}

#[test]
fn test_ongoing_outage_counts_up_to_now() {
    let outages = [outage("2026-10-20 10:00:00", None)];
    let days = daily_uptime(&window("2026-10-20 12:00:00", Tz::UTC, &outages, &[]));
    let today = day(&days, "2026-10-20");
    assert_eq!(today.down_seconds, 7200);
    assert_eq!(
        today.uptime_basis_points,
        Some((43_200 - 7200) * 10_000 / 43_200),
        "today only counts from midnight up to now"
    );
}

#[test]
fn test_days_before_the_site_was_added_are_not_monitored() {
    let mut window = window("2026-10-20 12:00:00", Tz::UTC, &[], &[]);
    window.monitored_from = utc("2026-10-18 12:00:00");
    let days = daily_uptime(&window);
    let monitoring_started: Vec<bool> = days.iter().map(|day| day.monitoring_started).collect();
    assert_eq!(
        monitoring_started,
        [false, false, false, false, true, true, true]
    );
    assert_eq!(day(&days, "2026-10-17").uptime_basis_points, None);
    assert_eq!(day(&days, "2026-10-18").uptime_basis_points, Some(10_000));
}

#[test]
fn test_window_summary_covers_only_the_monitored_part() {
    let outages = [outage("2026-10-20 06:00:00", Some("2026-10-20 12:00:00"))];
    let mut window = window("2026-10-21 00:00:00", Tz::UTC, &outages, &[]);
    window.monitored_from = utc("2026-10-20 00:00:00");
    let uptime = window_summary(&window);
    assert_eq!(uptime.uptime_basis_points, Some(7500));
    assert_eq!(uptime.unwatched_seconds, 0);
}

#[test]
fn test_canary_segments_follow_the_order_things_happened() {
    let other_host = "https://www.cloudflare.com";
    let periods = [
        canary(
            CanaryHealth::Healthy,
            other_host,
            "2026-10-20 09:00:00",
            Some("2026-10-20 10:00:00"),
        ),
        canary(
            CanaryHealth::Failing,
            other_host,
            "2026-10-20 10:00:00",
            Some("2026-10-20 10:30:00"),
        ),
        canary(
            CanaryHealth::Healthy,
            other_host,
            "2026-10-20 10:30:00",
            Some("2026-10-20 11:00:00"),
        ),
        canary(
            CanaryHealth::Healthy,
            other_host,
            "2026-10-20 11:00:00",
            None,
        ),
    ];
    let mut window = window("2026-10-20 12:00:00", Tz::UTC, &[], &periods);
    window.monitored_from = utc("2026-10-20 08:00:00");
    let days = daily_uptime(&window);
    let segments = segment_times(day(&days, "2026-10-20"));
    let expected = expected_segments(&[
        (WatchStatus::NoCanary, "08:00", "09:00"),
        (WatchStatus::CanaryHealthy, "09:00", "10:00"),
        (WatchStatus::CanaryFailing, "10:00", "10:30"),
        (WatchStatus::CanaryHealthy, "10:30", "12:00"),
    ]);
    assert_eq!(segments, expected, "back-to-back healthy periods merge");
}

#[test]
fn test_time_the_monitor_was_not_running_is_left_out_of_uptime() {
    let outages = [outage("2026-10-20 05:00:00", Some("2026-10-20 09:00:00"))];
    let monitoring_periods = [
        period("2026-10-20 00:00:00", "2026-10-20 06:00:00"),
        period("2026-10-20 08:00:00", "2026-10-20 11:59:30"),
    ];
    let mut window = window("2026-10-20 12:00:00", Tz::UTC, &outages, &[]);
    window.monitoring_periods = &monitoring_periods;
    let days = daily_uptime(&window);
    let today = day(&days, "2026-10-20");
    assert_eq!(today.watched_seconds, 10 * 3600);
    assert_eq!(
        today.down_seconds,
        2 * 3600,
        "only the outage hours either side of the gap count"
    );
    assert_eq!(today.uptime_basis_points, Some(8000));
    assert_eq!(today.not_running_seconds, 2 * 3600);
    assert_eq!(
        segment_times(today),
        expected_segments(&[
            (WatchStatus::NoCanary, "00:00", "06:00"),
            (WatchStatus::NotRunning, "06:00", "08:00"),
            (WatchStatus::NoCanary, "08:00", "12:00"),
        ]),
        "a heartbeat 30 seconds ago still counts as running now"
    );
}

#[test]
fn test_time_the_canary_could_not_connect_is_left_out_of_uptime() {
    let other_host = "https://www.cloudflare.com";
    let periods = [
        canary(
            CanaryHealth::Healthy,
            other_host,
            "2026-10-20 00:00:00",
            Some("2026-10-20 10:00:00"),
        ),
        canary(
            CanaryHealth::Failing,
            other_host,
            "2026-10-20 10:00:00",
            Some("2026-10-20 11:00:00"),
        ),
        canary(
            CanaryHealth::Healthy,
            other_host,
            "2026-10-20 11:00:00",
            None,
        ),
    ];
    let outages = [outage("2026-10-20 10:15:00", Some("2026-10-20 10:45:00"))];
    let window = window("2026-10-20 12:00:00", Tz::UTC, &outages, &periods);
    let days = daily_uptime(&window);
    let today = day(&days, "2026-10-20");
    assert_eq!(today.watched_seconds, 11 * 3600);
    assert_eq!(today.down_seconds, 0);
    assert_eq!(today.uptime_basis_points, Some(10_000));
    assert_eq!(today.outage_count, 1);
    assert_eq!(window_summary(&window).unwatched_seconds, 3600);
}

#[test]
fn test_day_with_nothing_watched_has_no_uptime() {
    let window = UptimeWindow {
        monitoring_periods: &[],
        ..window("2026-10-20 12:00:00", Tz::UTC, &[], &[])
    };
    let days = daily_uptime(&window);
    let today = day(&days, "2026-10-20");
    assert!(today.monitoring_started);
    assert_eq!(today.uptime_basis_points, None);
    assert_eq!(today.not_running_seconds, 12 * 3600);
}
