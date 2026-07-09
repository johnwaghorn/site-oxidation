import { useState } from "react";
import type { UseMutationResult } from "@tanstack/react-query";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import {
  FormInput,
  FormSelect,
  SecretInput,
} from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { useAuth } from "../hooks/useAuth";
import {
  useTeamNotifications,
  useTestEmailNotification,
  useTestSlackNotification,
  useTestTeamsNotification,
  useUpdateTeamNotifications,
} from "../hooks/useTeamNotifications";
import type { components } from "../generated/schema";

type UserTeam = components["schemas"]["UserTeam"];
type TeamNotifications = components["schemas"]["TeamNotificationsResponse"];
type UpdateTeamNotificationsRequest =
  components["schemas"]["UpdateTeamNotificationsRequest"];
type SmtpTlsMode = components["schemas"]["SmtpTlsMode"];
type SuccessResponse = components["schemas"]["SuccessResponse"];
type TestNotificationMutation = UseMutationResult<
  SuccessResponse,
  Error,
  number
>;

function TeamNotificationsCard({
  team,
  defaultOpen,
}: {
  team: UserTeam;
  defaultOpen: boolean;
}) {
  const { data, isLoading, error } = useTeamNotifications(team.id);
  const testSlack = useTestSlackNotification();
  const testTeams = useTestTeamsNotification();
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const emailConfigured = Boolean(
    data?.smtp_host && data?.smtp_from_email && data?.smtp_to_email,
  );
  const enabledChannels = [
    data?.slack_webhook_url ? "Slack" : null,
    data?.microsoft_teams_webhook_url ? "Microsoft Teams" : null,
    emailConfigured ? "Email" : null,
  ].filter((channel): channel is string => channel !== null);
  const hasChannel = enabledChannels.length > 0;

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
            {hasChannel
              ? `${enabledChannels.join(", ")} alerts are enabled.`
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
            className={hasChannel ? "pill pill-success" : "pill pill-neutral"}
          >
            {hasChannel ? "Enabled" : "Disabled"}
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
                sendTest={testSlack}
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
                  sendTest={testTeams}
                  label="Microsoft Teams webhook URL"
                  placeholder="https://prod-00.westus.logic.azure.com/workflows/..."
                  savedWebhookUrl={data?.microsoft_teams_webhook_url ?? ""}
                  buildPayload={(url) => ({ microsoft_teams_webhook_url: url })}
                />
              </div>
              {data && (
                <div className="card-section">
                  <SmtpForm
                    key={`smtp:${smtpFingerprint(data)}`}
                    teamId={team.id}
                    settings={data}
                  />
                </div>
              )}
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
  sendTest,
  label,
  placeholder,
  savedWebhookUrl,
  buildPayload,
}: {
  teamId: number;
  channel: string;
  sendTest: TestNotificationMutation;
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
      <label className="field-label">
        <span className="field-label-text">{label}</span>
        <SecretInput
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
        <p className="form-success-text">{channel} webhook saved.</p>
      )}
      {sendTest.isError && <ErrorMessage error={sendTest.error} />}
      {sendTest.isSuccess && (
        <p className="form-success-text">{channel} test message sent.</p>
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
            disabled={sendTest.isPending}
            onClick={() => sendTest.mutate(teamId)}
          >
            {sendTest.isPending ? "Sending..." : "Send test message"}
          </button>
        )}
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

function listMissingFields(fields: string[]) {
  if (fields.length === 1) {
    return fields[0];
  }
  return `${fields.slice(0, -1).join(", ")} and ${fields[fields.length - 1]}`;
}

function smtpFingerprint(settings: TeamNotifications) {
  return [
    settings.smtp_host,
    settings.smtp_port,
    settings.smtp_tls_mode,
    settings.smtp_auth,
    settings.smtp_username,
    settings.smtp_password_set,
    settings.smtp_from_email,
    settings.smtp_to_email,
  ].join("|");
}

function SmtpForm({
  teamId,
  settings,
}: {
  teamId: number;
  settings: TeamNotifications;
}) {
  const updateSmtp = useUpdateTeamNotifications();
  const sendTest = useTestEmailNotification();
  const [host, setHost] = useState(settings.smtp_host ?? "");
  const [port, setPort] = useState(settings.smtp_port?.toString() ?? "");
  const [tlsMode, setTlsMode] = useState(settings.smtp_tls_mode);
  const [smtpAuth, setSmtpAuth] = useState(settings.smtp_auth);
  const [username, setUsername] = useState(settings.smtp_username ?? "");
  const [password, setPassword] = useState("");
  const [fromEmail, setFromEmail] = useState(settings.smtp_from_email ?? "");
  const [toEmail, setToEmail] = useState(settings.smtp_to_email ?? "");
  const [missingFields, setMissingFields] = useState<string[]>([]);
  const emailConfigured = Boolean(
    settings.smtp_host && settings.smtp_from_email && settings.smtp_to_email,
  );

  const findMissingFields = () => {
    const disablingEmail =
      host.trim() === "" && fromEmail.trim() === "" && toEmail.trim() === "";
    if (disablingEmail) {
      return [];
    }
    const missing = [];
    if (host.trim() === "") {
      missing.push("SMTP host");
    }
    if (fromEmail.trim() === "") {
      missing.push("from address");
    }
    if (toEmail.trim() === "") {
      missing.push("to address");
    }
    if (smtpAuth && username.trim() === "") {
      missing.push("username");
    }
    if (smtpAuth && password === "" && !settings.smtp_password_set) {
      missing.push("password");
    }
    return missing;
  };

  return (
    <form
      className="form-column"
      style={{ gap: "12px" }}
      onSubmit={(event) => {
        event.preventDefault();
        const missing = findMissingFields();
        setMissingFields(missing);
        if (missing.length > 0) {
          return;
        }
        const payload: UpdateTeamNotificationsRequest = {
          smtp_host: host,
          smtp_tls_mode: tlsMode,
          smtp_auth: smtpAuth,
          smtp_username: username,
          smtp_from_email: fromEmail,
          smtp_to_email: toEmail,
        };
        if (port !== "") {
          payload.smtp_port = Number(port);
        }
        if (password !== "") {
          payload.smtp_password = password;
        }
        updateSmtp.mutate({ teamId, payload });
      }}
    >
      <h3 style={{ margin: "0 0 6px 0", fontSize: "16px" }}>Email</h3>
      <p className="muted-text" style={{ margin: 0, fontSize: "14px" }}>
        Send alerts by email through your own SMTP server.
      </p>
      <div className="field-row">
        <label className="field-label">
          <span className="field-label-text">SMTP host</span>
          <FormInput
            value={host}
            onChange={(event) => setHost(event.target.value)}
            placeholder="smtp.waghorn.tech"
          />
        </label>
        <label className="field-label">
          <span className="field-label-text">Port</span>
          <FormInput
            type="number"
            min={1}
            max={65535}
            value={port}
            onChange={(event) => setPort(event.target.value)}
            placeholder="Default for the TLS mode"
          />
        </label>
        <label className="field-label">
          <span className="field-label-text">TLS mode</span>
          <FormSelect
            value={tlsMode}
            onChange={(event) => setTlsMode(event.target.value as SmtpTlsMode)}
          >
            <option value="starttls">STARTTLS</option>
            <option value="tls">TLS</option>
            <option value="none">None</option>
          </FormSelect>
        </label>
      </div>
      <div className="field-row">
        <label className="field-label">
          <span className="field-label-text">From address</span>
          <FormInput
            value={fromEmail}
            onChange={(event) => setFromEmail(event.target.value)}
            placeholder="alerts@waghorn.tech"
          />
        </label>
        <label className="field-label">
          <span className="field-label-text">To address</span>
          <FormInput
            value={toEmail}
            onChange={(event) => setToEmail(event.target.value)}
            placeholder="on-call@waghorn.tech"
          />
        </label>
      </div>
      <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
        <input
          type="checkbox"
          checked={smtpAuth}
          onChange={(event) => setSmtpAuth(event.target.checked)}
        />
        Sign in with a username and password
      </label>
      {smtpAuth && (
        <div className="field-row">
          <label className="field-label">
            <span className="field-label-text">Username</span>
            <FormInput
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              autoComplete="off"
            />
          </label>
          <label className="field-label">
            <span className="field-label-text">Password</span>
            <SecretInput
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder={
                settings.smtp_password_set
                  ? "Leave blank to keep the saved password"
                  : ""
              }
            />
          </label>
        </div>
      )}
      <p className="muted-text" style={{ margin: 0, fontSize: "14px" }}>
        Leave the host blank and save to disable email notifications for this
        team. Without a port, 465 is used for TLS, 587 for STARTTLS and 25 with
        TLS off.
      </p>
      {missingFields.length > 0 && (
        <p className="error-box">
          Fill in the {listMissingFields(missingFields)} to enable email
          notifications.
        </p>
      )}
      {updateSmtp.isError && <ErrorMessage error={updateSmtp.error} />}
      {updateSmtp.isSuccess && (
        <p className="form-success-text">Email settings saved.</p>
      )}
      {sendTest.isError && <ErrorMessage error={sendTest.error} />}
      {sendTest.isSuccess && (
        <p className="form-success-text">Test email sent.</p>
      )}
      <div style={{ display: "flex", gap: "10px", flexWrap: "wrap" }}>
        <button
          className="button-primary-action"
          type="submit"
          disabled={updateSmtp.isPending}
        >
          {updateSmtp.isPending ? "Saving..." : "Save email settings"}
        </button>
        {emailConfigured && (
          <button
            className="button-secondary-action"
            type="button"
            disabled={sendTest.isPending}
            onClick={() => sendTest.mutate(teamId)}
          >
            {sendTest.isPending ? "Sending..." : "Send test email"}
          </button>
        )}
        {emailConfigured && (
          <button
            className="button-secondary-action"
            type="button"
            disabled={updateSmtp.isPending}
            onClick={() => {
              setHost("");
              updateSmtp.mutate({ teamId, payload: { smtp_host: "" } });
            }}
          >
            Disable email
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
