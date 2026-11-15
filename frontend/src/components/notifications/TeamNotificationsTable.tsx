import { Link } from "react-router-dom";
import { Truncate } from "../ui/Truncate";
import type { components } from "../../generated/schema";

type TeamNotificationSummary = components["schemas"]["TeamNotificationSummary"];

interface TeamNotificationsTableProps {
  summaries: TeamNotificationSummary[];
}

function listEventsThatSendAlerts(summary: TeamNotificationSummary) {
  const events = [
    summary.notify_site_down && "down",
    summary.notify_site_recovered && "recovered",
    summary.notify_cert_expiring && "certificate expiring",
  ].filter(Boolean);
  if (events.length === 0) {
    return "Nothing";
  }
  const text = events.join(", ");
  return text.charAt(0).toUpperCase() + text.slice(1);
}

export function TeamNotificationsTable({
  summaries,
}: TeamNotificationsTableProps) {
  return (
    <table className="data-table">
      <thead>
        <tr className="table-header-row">
          <th className="table-cell-left">Team</th>
          <th className="table-cell-center">Slack</th>
          <th className="table-cell-center">Microsoft Teams</th>
          <th className="table-cell-center">Email</th>
          <th className="table-cell-left">Alerts for</th>
        </tr>
      </thead>
      <tbody>
        {summaries.map((summary) => (
          <TeamNotificationsRow key={summary.team_id} summary={summary} />
        ))}
      </tbody>
    </table>
  );
}

function TeamNotificationsRow({
  summary,
}: {
  summary: TeamNotificationSummary;
}) {
  const hasNoAlertChannel =
    !summary.slack_enabled &&
    !summary.microsoft_teams_enabled &&
    !summary.email_enabled;

  return (
    <tr className="table-row">
      <td className="table-cell" style={{ fontWeight: 500 }}>
        <span
          style={{
            display: "inline-flex",
            flexWrap: "wrap",
            alignItems: "center",
            gap: "8px",
          }}
        >
          <Link to={`/notifications/${summary.team_id}`}>
            <Truncate text={summary.team_name} maxWidth={280} />
          </Link>
          {hasNoAlertChannel && (
            <span
              className="badge"
              title="Nobody will be alerted when this team's sites go down"
              style={{
                backgroundColor: "var(--color-warning-bg)",
                color: "var(--color-warning-text)",
              }}
            >
              NO ALERTS
            </span>
          )}
        </span>
      </td>
      <td className="table-cell-center">
        <ChannelOnOffBadge isOn={summary.slack_enabled} />
      </td>
      <td className="table-cell-center">
        <ChannelOnOffBadge isOn={summary.microsoft_teams_enabled} />
      </td>
      <td className="table-cell-center">
        <ChannelOnOffBadge isOn={summary.email_enabled} />
      </td>
      <td className="table-cell muted-text" style={{ fontSize: "14px" }}>
        {listEventsThatSendAlerts(summary)}
      </td>
    </tr>
  );
}

function ChannelOnOffBadge({ isOn }: { isOn: boolean }) {
  return (
    <span
      className="badge"
      style={{
        backgroundColor: isOn
          ? "var(--color-success-bg)"
          : "var(--color-neutral-bg)",
        color: isOn ? "var(--color-success-text)" : "var(--color-neutral-text)",
      }}
    >
      {isOn ? "ON" : "OFF"}
    </span>
  );
}
