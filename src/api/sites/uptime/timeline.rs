use crate::api::sites::responses::{WatchSegment, WatchStatus};
use crate::canary::{CanaryHealth, CanaryPeriod};
use crate::monitoring::periods::{MAX_HEARTBEAT_GAP_SECONDS, MonitoringPeriod};
use jiff::Timestamp;
use sqlx::FromRow;

const FULL_UPTIME_BASIS_POINTS: i64 = 10_000;

#[derive(Debug, FromRow)]
pub(in crate::api::sites) struct OutageTimes {
    pub started_at: jiff_sqlx::Timestamp,
    pub ended_at: Option<jiff_sqlx::Timestamp>,
}

#[derive(Debug, FromRow)]
pub(in crate::api::sites) struct BlockedTimes {
    pub started_at: jiff_sqlx::Timestamp,
    pub ended_at: Option<jiff_sqlx::Timestamp>,
}

#[derive(Clone, Copy)]
pub(super) struct TimeRange {
    pub(super) start: Timestamp,
    pub(super) end: Timestamp,
}

impl TimeRange {
    pub(super) fn duration_seconds(&self) -> i64 {
        self.end.duration_since(self.start).as_secs().max(0)
    }

    pub(super) fn overlap_seconds(&self, start: Timestamp, end: Timestamp) -> i64 {
        let overlap = TimeRange {
            start: self.start.max(start),
            end: self.end.min(end),
        };
        overlap.duration_seconds()
    }
}

pub(super) struct UptimeRecords<'a> {
    pub(super) running_ranges: &'a [TimeRange],
    pub(super) blocked_ranges: &'a [TimeRange],
    pub(super) canary_periods: &'a [CanaryPeriod],
    pub(super) outages: &'a [OutageTimes],
    pub(super) now: Timestamp,
}

pub(super) struct MeasuredUptime {
    pub(super) timeline: Vec<WatchSegment>,
    pub(super) blocked: Vec<TimeRange>,
    pub(super) watched: WatchedTime,
}

pub(super) struct WatchedTime {
    pub(super) watched_seconds: i64,
    pub(super) down_seconds: i64,
}

impl WatchedTime {
    pub(super) fn uptime_basis_points(&self) -> Option<i64> {
        uptime_as_basis_points(
            self.watched_seconds.saturating_sub(self.down_seconds),
            self.watched_seconds,
        )
    }
}

pub(super) fn monitor_running_ranges(
    monitoring_periods: &[MonitoringPeriod],
    now: Timestamp,
) -> Vec<TimeRange> {
    monitoring_periods
        .iter()
        .map(|period| {
            let last_seen_at = period.last_seen_at.to_jiff();
            let still_running =
                now.duration_since(last_seen_at).as_secs() <= MAX_HEARTBEAT_GAP_SECONDS;
            TimeRange {
                start: period.started_at.to_jiff(),
                end: if still_running { now } else { last_seen_at },
            }
        })
        .collect()
}

pub(super) fn site_blocked_ranges(
    blocked_periods: &[BlockedTimes],
    now: Timestamp,
) -> Vec<TimeRange> {
    blocked_periods
        .iter()
        .map(|period| TimeRange {
            start: period.started_at.to_jiff(),
            end: period.ended_at.map_or(now, jiff_sqlx::Timestamp::to_jiff),
        })
        .collect()
}

// The one place that decides what counts as watched, so every figure follows the same rules.
pub(super) fn measure_uptime_in_range(
    range: &TimeRange,
    records: &UptimeRecords<'_>,
    canary_shares_site_host: impl Fn(&str) -> bool,
) -> MeasuredUptime {
    let timeline = build_watch_timeline(
        range,
        records.running_ranges,
        records.canary_periods,
        records.now,
        canary_shares_site_host,
    );
    let blocked = blocked_time_while_monitor_ran(&timeline, records.blocked_ranges);
    let watched = measure_watched_time(&timeline, &blocked, records.outages, records.now);
    MeasuredUptime {
        timeline,
        blocked,
        watched,
    }
}

fn build_watch_timeline(
    range: &TimeRange,
    running_ranges: &[TimeRange],
    canary_periods: &[CanaryPeriod],
    now: Timestamp,
    canary_shares_site_host: impl Fn(&str) -> bool,
) -> Vec<WatchSegment> {
    let canary_only =
        segments_from_canary_periods(range, canary_periods, now, canary_shares_site_host);
    mark_time_monitor_was_not_running(&canary_only, running_ranges)
}

fn blocked_time_while_monitor_ran(
    timeline: &[WatchSegment],
    blocked_ranges: &[TimeRange],
) -> Vec<TimeRange> {
    let mut blocked: Vec<TimeRange> = Vec::new();
    for segment in timeline {
        if segment.state == WatchStatus::NotRunning {
            continue;
        }
        for blocked_range in blocked_ranges {
            let start = blocked_range.start.max(segment.started_at);
            let end = blocked_range.end.min(segment.ended_at);
            if end <= start {
                continue;
            }
            if let Some(previous) = blocked.last_mut()
                && previous.end == start
            {
                previous.end = end;
            } else {
                blocked.push(TimeRange { start, end });
            }
        }
    }
    blocked
}

pub(super) fn total_seconds(ranges: &[TimeRange]) -> i64 {
    ranges.iter().fold(0_i64, |total, range| {
        total.saturating_add(range.duration_seconds())
    })
}

fn measure_watched_time(
    timeline: &[WatchSegment],
    blocked: &[TimeRange],
    outages: &[OutageTimes],
    now: Timestamp,
) -> WatchedTime {
    let watched: Vec<TimeRange> = timeline
        .iter()
        .filter(|segment| {
            !matches!(
                segment.state,
                WatchStatus::CanaryFailing | WatchStatus::NotRunning
            )
        })
        .flat_map(|segment| {
            time_outside(
                TimeRange {
                    start: segment.started_at,
                    end: segment.ended_at,
                },
                blocked,
            )
        })
        .collect();
    let down_seconds = outages.iter().fold(0_i64, |total, outage| {
        let outage_end = outage.ended_at.map_or(now, jiff_sqlx::Timestamp::to_jiff);
        watched.iter().fold(total, |total, range| {
            total.saturating_add(range.overlap_seconds(outage.started_at.to_jiff(), outage_end))
        })
    });
    WatchedTime {
        watched_seconds: total_seconds(&watched),
        down_seconds,
    }
}

pub(super) fn seconds_in_state(timeline: &[WatchSegment], state: WatchStatus) -> i64 {
    timeline
        .iter()
        .filter(|segment| segment.state == state)
        .fold(0_i64, |total, segment| {
            total.saturating_add(
                segment
                    .ended_at
                    .duration_since(segment.started_at)
                    .as_secs(),
            )
        })
}

fn uptime_as_basis_points(up_seconds: i64, total_seconds: i64) -> Option<i64> {
    if total_seconds <= 0 {
        return None;
    }
    up_seconds
        .max(0)
        .saturating_mul(FULL_UPTIME_BASIS_POINTS)
        .checked_div(total_seconds)
}

fn segments_from_canary_periods(
    range: &TimeRange,
    periods: &[CanaryPeriod],
    now: Timestamp,
    canary_shares_site_host: impl Fn(&str) -> bool,
) -> Vec<WatchSegment> {
    let mut segments: Vec<WatchSegment> = Vec::new();
    let mut covered_until = range.start;
    for period in periods {
        let started_at = period.started_at.to_jiff().max(covered_until);
        let ended_at = period
            .ended_at
            .map_or(now, jiff_sqlx::Timestamp::to_jiff)
            .min(range.end);
        if ended_at <= started_at {
            continue;
        }
        if started_at > covered_until {
            push_segment_or_extend_previous(
                &mut segments,
                WatchStatus::NoCanary,
                covered_until,
                started_at,
            );
        }
        let state = if period.state == CanaryHealth::Failing {
            WatchStatus::CanaryFailing
        } else if canary_shares_site_host(&period.url) {
            WatchStatus::CanarySharesHost
        } else {
            WatchStatus::CanaryHealthy
        };
        push_segment_or_extend_previous(&mut segments, state, started_at, ended_at);
        covered_until = ended_at;
    }
    if range.end > covered_until {
        push_segment_or_extend_previous(
            &mut segments,
            WatchStatus::NoCanary,
            covered_until,
            range.end,
        );
    }
    segments
}

fn mark_time_monitor_was_not_running(
    canary_only: &[WatchSegment],
    running_ranges: &[TimeRange],
) -> Vec<WatchSegment> {
    let mut segments: Vec<WatchSegment> = Vec::new();
    for segment in canary_only {
        let mut covered_until = segment.started_at;
        for running_range in running_ranges {
            let started_at = running_range.start.max(covered_until);
            let ended_at = running_range.end.min(segment.ended_at);
            if ended_at <= started_at {
                continue;
            }
            if started_at > covered_until {
                push_segment_or_extend_previous(
                    &mut segments,
                    WatchStatus::NotRunning,
                    covered_until,
                    started_at,
                );
            }
            push_segment_or_extend_previous(&mut segments, segment.state, started_at, ended_at);
            covered_until = ended_at;
        }
        if segment.ended_at > covered_until {
            push_segment_or_extend_previous(
                &mut segments,
                WatchStatus::NotRunning,
                covered_until,
                segment.ended_at,
            );
        }
    }
    segments
}

fn time_outside(range: TimeRange, excluded: &[TimeRange]) -> Vec<TimeRange> {
    let mut remaining: Vec<TimeRange> = Vec::new();
    let mut covered_until = range.start;
    for excluded_range in excluded {
        let start = excluded_range.start.max(covered_until);
        let end = excluded_range.end.min(range.end);
        if end <= start {
            continue;
        }
        if start > covered_until {
            remaining.push(TimeRange {
                start: covered_until,
                end: start,
            });
        }
        covered_until = end;
    }
    if range.end > covered_until {
        remaining.push(TimeRange {
            start: covered_until,
            end: range.end,
        });
    }
    remaining
}

fn push_segment_or_extend_previous(
    segments: &mut Vec<WatchSegment>,
    state: WatchStatus,
    started_at: Timestamp,
    ended_at: Timestamp,
) {
    if let Some(previous) = segments.last_mut()
        && previous.state == state
        && previous.ended_at == started_at
    {
        previous.ended_at = ended_at;
        return;
    }
    segments.push(WatchSegment {
        state,
        started_at,
        ended_at,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uptime_is_rounded_down_never_up_to_100_percent() {
        assert_eq!(uptime_as_basis_points(86_370, 86_400), Some(9996));
        assert_eq!(uptime_as_basis_points(9_999_999, 10_000_000), Some(9999));
        assert_eq!(uptime_as_basis_points(0, 0), None);
    }
}
