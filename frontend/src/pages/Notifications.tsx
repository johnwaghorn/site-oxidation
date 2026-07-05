import { useState } from "react";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { FormInput } from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { useAuth } from "../hooks/useAuth";
import {
  useTeamNotifications,
  useUpdateTeamNotifications,
} from "../hooks/useTeamNotifications";
import type { components } from "../generated/schema";

type UserTeam = components["schemas"]["UserTeam"];
type TeamNotifications = components["schemas"]["TeamNotificationsResponse"];
type UpdateTeamNotificationsRequest =
  components["schemas"]["UpdateTeamNotificationsRequest"];

function TeamNotificationsCard({
  team,
  defaultOpen,
}: {
  team: UserTeam;
  defaultOpen: boolean;
}) {
  const { data, isLoading, error } = useTeamNotifications(team.id);
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const enabledChannels = [
    data?.slack_webhook_url ? "Slack" : null,
    data?.microsoft_teams_webhook_url ? "Microsoft Teams" : null,
  ].filter((channel): channel is string => channel !== null);
  const hasWebhook = enabledChannels.length > 0;

  return (
    <section className="card">
      <button
        type="button"
        className="card-disclosure"
        aria-expanded={isOpen}
        onClick={() => setIsOpen((open) => !open)}
      >
        <span>
          <span
            style={{
              display: "block",
              margin: "0 0 6px 0",
              fontSize: "22px",
              fontWeight: 700,
            }}
          >
            {team.name}
          </span>
          <span className="muted-text" style={{ display: "block" }}>
            {hasWebhook
              ? `${enabledChannels.join(" and ")} alerts are enabled.`
              : "No notifications configured."}
          </span>
        </span>
        <span
          style={{
            display: "flex",
            alignItems: "center",
            gap: "10px",
            flexShrink: 0,
          }}
        >
          <span
            className={hasWebhook ? "pill pill-success" : "pill pill-neutral"}
          >
            {hasWebhook ? "Enabled" : "Disabled"}
          </span>
          <span
            aria-hidden="true"
            className="card-disclosure-chevron muted-text"
            style={{
              fontSize: "14px",
              transform: isOpen ? "none" : "rotate(-90deg)",
            }}
          >
            ▾
          </span>
        </span>
      </button>

      {isOpen && (
        <div style={{ marginTop: "18px" }}>
          {isLoading ? (
            <LoadingSpinner />
          ) : error ? (
            <ErrorMessage error={error} />
          ) : (
            <>
              <WebhookForm
                key={`slack:${data?.slack_webhook_url ?? ""}`}
                teamId={team.id}
                channel="Slack"
                label="Slack webhook URL"
                placeholder="https://hooks.slack.com/services/..."
                savedWebhookUrl={data?.slack_webhook_url ?? ""}
                buildPayload={(url) => ({ slack_webhook_url: url })}
              />
              <div className="card-section">
                <WebhookForm
                  key={`teams:${data?.microsoft_teams_webhook_url ?? ""}`}
                  teamId={team.id}
                  channel="Microsoft Teams"
                  label="Microsoft Teams webhook URL"
                  placeholder="https://prod-00.westus.logic.azure.com/workflows/..."
                  savedWebhookUrl={data?.microsoft_teams_webhook_url ?? ""}
                  buildPayload={(url) => ({ microsoft_teams_webhook_url: url })}
                />
              </div>
              {data && (
                <NotificationEventToggles teamId={team.id} settings={data} />
              )}
            </>
          )}
        </div>
      )}
    </section>
  );
}

const notificationEvents: {
  key: "notify_site_down" | "notify_site_recovered" | "notify_cert_expiring";
  label: string;
  payload: (enabled: boolean) => UpdateTeamNotificationsRequest;
}[] = [
  {
    key: "notify_site_down",
    label: "Site down",
    payload: (enabled) => ({ notify_site_down: enabled }),
  },
  {
    key: "notify_site_recovered",
    label: "Site recovered",
    payload: (enabled) => ({ notify_site_recovered: enabled }),
  },
  {
    key: "notify_cert_expiring",
    label: "Certificate expiring",
    payload: (enabled) => ({ notify_cert_expiring: enabled }),
  },
];

function NotificationEventToggles({
  teamId,
  settings,
}: {
  teamId: number;
  settings: TeamNotifications;
}) {
  const updateEvents = useUpdateTeamNotifications();

  return (
    <div className="card-section">
      <h3 style={{ margin: "0 0 6px 0", fontSize: "16px" }}>Alert events</h3>
      <p
        className="muted-text"
        style={{ margin: "0 0 12px 0", fontSize: "14px" }}
      >
        Choose which events send an alert to this team.
      </p>
      <div style={{ display: "flex", gap: "18px", flexWrap: "wrap" }}>
        {notificationEvents.map(({ key, label, payload }) => (
          <label
            key={key}
            style={{ display: "flex", alignItems: "center", gap: "8px" }}
          >
            <input
              type="checkbox"
              checked={settings[key]}
              disabled={updateEvents.isPending}
              onChange={(event) =>
                updateEvents.mutate({
                  teamId,
                  payload: payload(event.target.checked),
                })
              }
            />
            {label}
          </label>
        ))}
      </div>
      {updateEvents.isError && <ErrorMessage error={updateEvents.error} />}
    </div>
  );
}

function WebhookForm({
  teamId,
  channel,
  label,
  placeholder,
  savedWebhookUrl,
  buildPayload,
}: {
  teamId: number;
  channel: string;
  label: string;
  placeholder: string;
  savedWebhookUrl: string;
  buildPayload: (url: string) => UpdateTeamNotificationsRequest;
}) {
  const updateWebhook = useUpdateTeamNotifications();
  const [webhookUrl, setWebhookUrl] = useState(savedWebhookUrl);
  const hasWebhook = Boolean(savedWebhookUrl);

  return (
    <form
      className="form-column"
      style={{ gap: "12px" }}
      onSubmit={(event) => {
        event.preventDefault();
        updateWebhook.mutate({
          teamId,
          payload: buildPayload(webhookUrl),
        });
      }}
    >
      <label style={{ display: "grid", gap: "8px" }}>
        <span style={{ fontWeight: 700 }}>{label}</span>
        <FormInput
          type="url"
          value={webhookUrl}
          onChange={(event) => setWebhookUrl(event.target.value)}
          placeholder={placeholder}
          style={{ width: "100%" }}
        />
      </label>
      <p className="muted-text" style={{ margin: 0, fontSize: "14px" }}>
        Leave this blank and save to disable {channel} notifications for this
        team.
      </p>
      {updateWebhook.isError && <ErrorMessage error={updateWebhook.error} />}
      {updateWebhook.isSuccess && (
        <p
          style={{
            margin: 0,
            color: "var(--color-success-text)",
            fontWeight: 700,
          }}
        >
          {channel} webhook saved.
        </p>
      )}
      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        <button
          className="button-primary-action"
          type="submit"
          disabled={updateWebhook.isPending}
        >
          {updateWebhook.isPending ? "Saving..." : "Save webhook"}
        </button>
        {hasWebhook && (
          <button
            className="button-secondary-action"
            type="button"
            disabled={updateWebhook.isPending}
            onClick={() => {
              setWebhookUrl("");
              updateWebhook.mutate({
                teamId,
                payload: buildPayload(""),
              });
            }}
          >
            Disable {channel}
          </button>
        )}
      </div>
    </form>
  );
}

export function Notifications() {
  const { teams, isLoading } = useAuth();

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Notifications</h1>
      <p
        className="muted-text"
        style={{ maxWidth: "680px", margin: "0 0 24px 0" }}
      >
        Configure notifications for each team you can access and choose which
        events trigger them. Every monitored site assigned to that team will use
        the same notification configuration.
      </p>

      {isLoading ? (
        <LoadingSpinner />
      ) : teams.length === 0 ? (
        <section className="card-hero" style={{ maxWidth: "680px" }}>
          <h2>No teams available</h2>
          <p className="muted-text" style={{ maxWidth: "520px", margin: 0 }}>
            Join or create a team before configuring notifications.
          </p>
        </section>
      ) : (
        <div
          style={{
            display: "grid",
            gap: "16px",
            maxWidth: "760px",
          }}
        >
          {teams.map((team) => (
            <TeamNotificationsCard
              key={team.id}
              team={team}
              defaultOpen={teams.length === 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}
