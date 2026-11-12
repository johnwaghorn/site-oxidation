use crate::api::sites::responses::{WatchSegment, WatchStatus};
use crate::canary::{CanaryHealth, CanaryPeriod};
use crate::monitoring::periods::{MAX_HEARTBEAT_GAP_SECONDS, MonitoringPeriod};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

const FULL_UPTIME_BASIS_POINTS: i64 = 10_000;

#[derive(Debug, FromRow)]
pub(in crate::api::sites) struct OutageTimes {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy)]
pub(super) struct TimeRange {
    pub(super) start: DateTime<Utc>,
    pub(super) end: DateTime<Utc>,
}

impl TimeRange {
    pub(super) fn duration_seconds(&self) -> i64 {
        self.end
            .signed_duration_since(self.start)
            .num_seconds()
            .max(0)
    }

    pub(super) fn overlap_seconds(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> i64 {
        let overlap = TimeRange {
            start: self.start.max(start),
            end: self.end.min(end),
        };
        overlap.duration_seconds()
    }
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
    now: DateTime<Utc>,
) -> Vec<TimeRange> {
    monitoring_periods
        .iter()
        .map(|period| {
            let still_running = now.signed_duration_since(period.last_seen_at).num_seconds()
                <= MAX_HEARTBEAT_GAP_SECONDS;
            TimeRange {
                start: period.started_at,
                end: if still_running {
                    now
                } else {
                    period.last_seen_at
                },
            }
        })
        .collect()
}

pub(super) fn build_watch_timeline(
    range: &TimeRange,
    running_ranges: &[TimeRange],
    canary_periods: &[CanaryPeriod],
    now: DateTime<Utc>,
    canary_shares_site_host: impl Fn(&str) -> bool,
) -> Vec<WatchSegment> {
    let canary_only =
        segments_from_canary_periods(range, canary_periods, now, canary_shares_site_host);
    mark_time_monitor_was_not_running(&canary_only, running_ranges)
}

pub(super) fn measure_watched_time(
    timeline: &[WatchSegment],
    outages: &[OutageTimes],
    now: DateTime<Utc>,
) -> WatchedTime {
    let watched: Vec<TimeRange> = timeline
        .iter()
        .filter(|segment| {
            !matches!(
                segment.state,
                WatchStatus::CanaryFailing | WatchStatus::NotRunning
            )
        })
        .map(|segment| TimeRange {
            start: segment.started_at,
            end: segment.ended_at,
        })
        .collect();
    let down_seconds = outages.iter().fold(0_i64, |total, outage| {
        let outage_end = outage.ended_at.unwrap_or(now);
        watched.iter().fold(total, |total, range| {
            total.saturating_add(range.overlap_seconds(outage.started_at, outage_end))
        })
    });
    WatchedTime {
        watched_seconds: watched.iter().fold(0_i64, |total, range| {
            total.saturating_add(range.duration_seconds())
        }),
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
                    .signed_duration_since(segment.started_at)
                    .num_seconds(),
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
    now: DateTime<Utc>,
    canary_shares_site_host: impl Fn(&str) -> bool,
) -> Vec<WatchSegment> {
    let mut segments: Vec<WatchSegment> = Vec::new();
    let mut covered_until = range.start;
    for period in periods {
        let started_at = period.started_at.max(covered_until);
        let ended_at = period.ended_at.unwrap_or(now).min(range.end);
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

fn push_segment_or_extend_previous(
    segments: &mut Vec<WatchSegment>,
    state: WatchStatus,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
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
