import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useSiteUptime, type UptimeWindowDays } from "../../hooks/useSites";
import { formatDuration, formatUptime } from "../../lib/uptime";
import { ErrorMessage } from "../ui/ErrorMessage";
import { LoadingSpinner } from "../ui/LoadingSpinner";
import type { components } from "../../generated/schema";

type UptimeDay = components["schemas"]["UptimeDay"];
type WatchStatus = components["schemas"]["WatchStatus"];
type WatchSegment = components["schemas"]["WatchSegment"];

interface TimeShare {
  className: string;
  label: string;
  seconds: number;
}

const WATCH_STATE_LABELS: Record<WatchStatus, string> = {
  canary_healthy: "Canary watching",
  canary_shares_host: "Canary checks this site",
  canary_failing: "Canary couldn't connect",
  no_canary: "No canary",
  not_running: "Monitor not running",
};

const CANARY_LEGEND_LABELS: Record<WatchStatus, string> = {
  canary_healthy: "Watching",
  canary_shares_host: "Checks this site",
  canary_failing: "Couldn't connect",
  no_canary: "No canary",
  not_running: "Monitor not running",
};

const WINDOW_DAY_LABELS: Record<UptimeWindowDays, string> = {
  7: "7 days",
  30: "30 days",
  90: "90 days",
};
const WINDOW_DAY_OPTIONS = Object.keys(WINDOW_DAY_LABELS).map(
  Number,
) as UptimeWindowDays[];
const LONG_OUTAGE_MINUTES = 15;

function outageClassName(day: UptimeDay) {
  if (day.down_seconds >= LONG_OUTAGE_MINUTES * 60) return "is-long";
  return day.down_seconds > 0 ? "is-short" : null;
}

function barClassName(day: UptimeDay, isDownNow: boolean) {
  const classes = ["uptime-bar", outageClassName(day)];
  if (isDownNow) classes.push("is-down-now");
  return classes.filter(Boolean).join(" ");
}

function describeOutages(day: UptimeDay, downSince: string | null) {
  if (downSince) {
    const soFar = Math.floor(
      (Date.now() - new Date(downSince).getTime()) / 1000,
    );
    return `Down now, ${formatDuration(soFar)} so far`;
  }
  if (day.outage_count === 0) return "No outages";
  const outages = day.outage_count === 1 ? "outage" : "outages";
  return `${day.outage_count} ${outages}, ${formatDuration(day.down_seconds)} down`;
}

function secondsByWatchState(day: UptimeDay) {
  const secondsByState: [WatchStatus, number][] = [
    ["canary_healthy", day.canary_healthy_seconds],
    ["canary_shares_host", day.canary_shares_host_seconds],
    ["canary_failing", day.canary_failing_seconds],
    ["no_canary", day.no_canary_seconds],
    ["not_running", day.not_running_seconds],
  ];
  return secondsByState.filter(([, seconds]) => seconds > 0);
}

function describeWatchStates(day: UptimeDay) {
  const parts = secondsByWatchState(day);
  if (parts.length === 0) return null;
  if (parts.length === 1) return `${WATCH_STATE_LABELS[parts[0][0]]} all day`;
  return parts
    .map(
      ([state, seconds]) =>
        `${WATCH_STATE_LABELS[state]} ${formatDuration(seconds)}`,
    )
    .join(", ");
}

function localDayBounds(date: string) {
  const [year, month, day] = date.split("-").map(Number);
  return {
    start: new Date(year, month - 1, day).getTime(),
    end: new Date(year, month - 1, day + 1).getTime(),
  };
}

function segmentPositionInDay(segment: WatchSegment, date: string) {
  const day = localDayBounds(date);
  const length = day.end - day.start;
  const start = new Date(segment.started_at).getTime() - day.start;
  const end = new Date(segment.ended_at).getTime() - day.start;
  return {
    left: `${(start / length) * 100}%`,
    width: `${((end - start) / length) * 100}%`,
  };
}

function isWatched(segment: WatchSegment) {
  return segment.state !== "not_running" && segment.state !== "canary_failing";
}

function moveFocusWithArrowKeys(event: KeyboardEvent<HTMLElement>) {
  const neighbour =
    event.key === "ArrowLeft"
      ? event.currentTarget.previousElementSibling
      : event.key === "ArrowRight"
        ? event.currentTarget.nextElementSibling
        : null;
  if (neighbour instanceof HTMLElement) {
    event.preventDefault();
    neighbour.focus();
  }
}

function describeDayUptime(day: UptimeDay) {
  if (day.uptime_basis_points == null) return "Not watched, so no uptime";
  const uptime = `${formatUptime(day.uptime_basis_points)} uptime`;
  const partlyUnwatched =
    day.not_running_seconds > 0 || day.canary_failing_seconds > 0;
  return partlyUnwatched
    ? `${uptime} over ${formatDuration(day.watched_seconds)} watched`
    : uptime;
}

function formatDayDate(day: UptimeDay) {
  return new Date(`${day.date}T00:00:00`).toLocaleDateString(undefined, {
    weekday: "short",
    day: "numeric",
    month: "short",
    year: "numeric",
  });
}

function describeDay(day: UptimeDay, downSince: string | null) {
  if (!day.monitoring_started) return "Not monitored yet";
  return [
    describeDayUptime(day),
    describeOutages(day, downSince),
    describeWatchStates(day),
  ]
    .filter((line) => line !== null)
    .join(". ");
}

function siteTimeShares(day: UptimeDay): TimeShare[] {
  const outages =
    day.outage_count === 1 ? "1 outage" : `${day.outage_count} outages`;
  return [
    {
      className: "is-up",
      label: "Up",
      seconds: day.watched_seconds - day.down_seconds,
    },
    {
      className: outageClassName(day) ?? "is-short",
      label: `Down, ${outages}`,
      seconds: day.down_seconds,
    },
  ].filter((share) => share.seconds > 0);
}

function canaryTimeShares(day: UptimeDay): TimeShare[] {
  return secondsByWatchState(day).map(([state, seconds]) => ({
    className: `is-${state}`,
    label: CANARY_LEGEND_LABELS[state],
    seconds,
  }));
}

function TooltipSection({
  title,
  shares,
}: {
  title: string;
  shares: TimeShare[];
}) {
  if (shares.length === 0) return null;
  return (
    <div className="uptime-tooltip-section">
      <span className="uptime-tooltip-title">{title}</span>
      <div className="uptime-tooltip-bar">
        {shares.map((share) => (
          <span
            key={share.label}
            className={share.className}
            style={{ flexGrow: share.seconds }}
          />
        ))}
      </div>
      {shares.map((share) => (
        <div key={share.label} className="uptime-tooltip-row">
          <span className={`uptime-swatch ${share.className}`} />
          <span>{share.label}</span>
          <span className="uptime-tooltip-duration">
            {formatDuration(share.seconds)}
          </span>
        </div>
      ))}
    </div>
  );
}

function DayTooltip({
  day,
  downSince,
  left,
}: {
  day: UptimeDay;
  downSince: string | null;
  left: string;
}) {
  const hasNoOutages =
    day.uptime_basis_points != null && day.outage_count === 0 && !downSince;
  return (
    <div className="uptime-tooltip" style={{ left }} aria-hidden="true">
      <div className="uptime-tooltip-heading">
        <strong>{formatDayDate(day)}</strong>
        {day.monitoring_started ? (
          <span className="muted-text">
            {describeDayUptime(day)}
            {hasNoOutages && ", no outages"}
          </span>
        ) : (
          <span className="muted-text">Not monitored yet</span>
        )}
        {downSince && (
          <span className="uptime-tooltip-down-now">
            {describeOutages(day, downSince)}
          </span>
        )}
      </div>
      {day.monitoring_started && (
        <>
          <TooltipSection title="Site" shares={siteTimeShares(day)} />
          <TooltipSection title="Canary" shares={canaryTimeShares(day)} />
        </>
      )}
    </div>
  );
}

export function UptimeHistory({ siteId }: { siteId: number }) {
  const [days, setDays] = useState<UptimeWindowDays>(90);
  const [activeIndex, setActiveIndex] = useState<number | null>(null);
  const sectionRef = useRef<HTMLElement>(null);
  const { data, isLoading, error } = useSiteUptime(siteId, days);

  useEffect(() => {
    if (window.location.hash === "#uptime") {
      sectionRef.current?.scrollIntoView();
    }
  }, []);

  const lastIndex = data ? data.days.length - 1 : -1;
  const monitoredDays = data
    ? data.days.filter((day) => day.monitoring_started).length
    : 0;
  const hoveredDay =
    data && activeIndex !== null ? data.days[activeIndex] : undefined;

  return (
    <section id="uptime" ref={sectionRef} className="card uptime-card">
      <div className="uptime-header">
        <div>
          <h2>Uptime</h2>
          <p className="muted-text" style={{ margin: 0 }}>
            {data?.uptime_basis_points != null ? (
              <>
                <strong className="uptime-total">
                  {formatUptime(data.uptime_basis_points)}
                </strong>{" "}
                over the last{" "}
                {monitoredDays === 1 ? "day" : `${monitoredDays} days`}
                {data.unwatched_seconds > 0 &&
                  ` · ${formatDuration(data.unwatched_seconds)} not watched`}
              </>
            ) : (
              "No uptime recorded yet"
            )}
          </p>
        </div>
        <div className="uptime-range" role="group" aria-label="Uptime period">
          {WINDOW_DAY_OPTIONS.map((option) => (
            <button
              key={option}
              type="button"
              aria-pressed={option === days}
              onClick={() => setDays(option)}
            >
              {WINDOW_DAY_LABELS[option]}
            </button>
          ))}
        </div>
      </div>

      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : data ? (
        <>
          <div
            className="uptime-chart"
            data-days={data.days.length}
            onMouseLeave={() => setActiveIndex(null)}
          >
            <div className="uptime-bars">
              {data.days.map((day, index) => {
                const isDownNow =
                  index === lastIndex && data.down_since != null;
                const description = describeDay(
                  day,
                  isDownNow ? (data.down_since ?? null) : null,
                );
                return (
                  <span
                    key={day.date}
                    role="img"
                    aria-label={`${formatDayDate(day)}: ${description}`}
                    className={barClassName(day, isDownNow)}
                    tabIndex={index === (activeIndex ?? lastIndex) ? 0 : -1}
                    onMouseEnter={() => setActiveIndex(index)}
                    onFocus={() => setActiveIndex(index)}
                    onBlur={() => setActiveIndex(null)}
                    onKeyDown={moveFocusWithArrowKeys}
                  >
                    {day.watch_segments.filter(isWatched).map((segment) => (
                      <span
                        key={segment.started_at}
                        className="uptime-bar-part"
                        style={segmentPositionInDay(segment, day.date)}
                      />
                    ))}
                  </span>
                );
              })}
            </div>
            <div className="uptime-canary-line" aria-hidden="true">
              {data.days.map((day) => (
                <span key={day.date} className="uptime-canary">
                  {day.watch_segments
                    .filter((segment) => segment.state !== "no_canary")
                    .map((segment) => (
                      <span
                        key={segment.started_at}
                        className={`uptime-canary-part is-${segment.state}`}
                        style={segmentPositionInDay(segment, day.date)}
                      />
                    ))}
                </span>
              ))}
            </div>
            {hoveredDay && activeIndex !== null && (
              <DayTooltip
                day={hoveredDay}
                downSince={
                  activeIndex === lastIndex ? (data.down_since ?? null) : null
                }
                left={`clamp(144px, ${((activeIndex + 0.5) / data.days.length) * 100}%, calc(100% - 144px))`}
              />
            )}
          </div>
          <div className="uptime-legend muted-text">
            <div className="uptime-legend-row">
              <span className="uptime-legend-label" id="uptime-legend-site">
                Site
              </span>
              <ul aria-labelledby="uptime-legend-site">
                <li>
                  <span className="uptime-swatch is-up" />
                  No outages
                </li>
                <li>
                  <span className="uptime-swatch is-short" />
                  Down under {LONG_OUTAGE_MINUTES} min
                </li>
                <li>
                  <span className="uptime-swatch is-long" />
                  Down {LONG_OUTAGE_MINUTES} min or more
                </li>
                <li>
                  <span className="uptime-swatch is-unmonitored" />
                  Not watched
                </li>
              </ul>
            </div>
            <div className="uptime-legend-row">
              <span className="uptime-legend-label" id="uptime-legend-canary">
                Canary
              </span>
              <ul aria-labelledby="uptime-legend-canary">
                <li>
                  <span className="uptime-swatch-line is-canary_healthy" />
                  {CANARY_LEGEND_LABELS.canary_healthy}
                </li>
                <li>
                  <span className="uptime-swatch-line is-canary_shares_host" />
                  {CANARY_LEGEND_LABELS.canary_shares_host}
                </li>
                <li>
                  <span className="uptime-swatch-line is-canary_failing" />
                  {CANARY_LEGEND_LABELS.canary_failing}
                </li>
                <li>
                  <span className="uptime-swatch-line is-not_running" />
                  {CANARY_LEGEND_LABELS.not_running}
                </li>
              </ul>
            </div>
          </div>
        </>
      ) : null}
    </section>
  );
}
