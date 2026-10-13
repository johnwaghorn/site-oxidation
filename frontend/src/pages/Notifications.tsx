import { useState } from "react";
import { Link, useParams } from "react-router-dom";
import type { UseMutationResult } from "@tanstack/react-query";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import {
  FormInput,
  FormSelect,
  SecretInput,
} from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Modal } from "../components/ui/Modal";
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
type Channel = "slack" | "teams" | "email";

function isEmailConfigured(settings: TeamNotifications | undefined) {
  return Boolean(
    settings?.smtp_host && settings.smtp_from_email && settings.smtp_to_email,
  );
}

function countChannelsOn(settings: TeamNotifications | undefined) {
  return [
    settings?.slack_webhook_url,
    settings?.microsoft_teams_webhook_url,
    isEmailConfigured(settings),
  ].filter(Boolean).length;
}

export function Notifications() {
  const { teams, isLoading } = useAuth();
  const { teamId } = useParams<{ teamId: string }>();
  const selected =
    teams.find((team) => team.id === Number(teamId)) ?? teams[0] ?? null;

  return (
    <div className="page-wrapper">
      <h1 className="page-title" style={{ marginBottom: "8px" }}>
        Notifications
      </h1>
      <p
        className="muted-text"
        style={{ maxWidth: "680px", margin: "0 0 24px" }}
      >
        Each team chooses where its alerts go. Every site in a team uses that
        team&apos;s settings.
      </p>

      {isLoading ? (
        <LoadingSpinner />
      ) : !selected ? (
        <section className="card-hero">
          <h2>No teams available</h2>
          <p className="muted-text" style={{ maxWidth: "520px", margin: 0 }}>
            Join or create a team before configuring notifications.
          </p>
        </section>
      ) : teams.length === 1 ? (
        <TeamSettings key={selected.id} team={selected} />
      ) : (
        <div className="notifications-layout">
          <nav aria-label="Teams" className="notification-team-list">
            {teams.map((team) => (
              <TeamListLink
                key={team.id}
                team={team}
                isSelected={team.id === selected.id}
              />
            ))}
          </nav>
          <TeamSettings key={selected.id} team={selected} />
        </div>
      )}
    </div>
  );
}

function TeamListLink({
  team,
  isSelected,
}: {
  team: UserTeam;
  isSelected: boolean;
}) {
  const { data } = useTeamNotifications(team.id);
  const channelsOn = countChannelsOn(data);

  return (
    <Link
      to={`/notifications/${team.id}`}
      aria-current={isSelected ? "page" : undefined}
    >
      <span className="notification-team-name">{team.name}</span>
      {data && (
        <span
          className={channelsOn > 0 ? "pill pill-success" : "pill pill-warning"}
        >
          {channelsOn > 0 ? `${channelsOn} on` : "No alerts"}
        </span>
      )}
    </Link>
  );
}

function TeamSettings({ team }: { team: UserTeam }) {
  const { data, isLoading, error } = useTeamNotifications(team.id);
  const testSlack = useTestSlackNotification();
  const testTeams = useTestTeamsNotification();
  const testEmail = useTestEmailNotification();
  const [editing, setEditing] = useState<Channel | null>(null);
  const closeDialog = () => setEditing(null);

  return (
    <section
      className="card notification-settings"
      aria-labelledby={`team-${team.id}`}
    >
      <h2 id={`team-${team.id}`} style={{ margin: "0 0 16px" }}>
        {team.name}
      </h2>
      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : data ? (
        <>
          {countChannelsOn(data) === 0 && (
            <p className="warning-box">
              Nobody will be alerted when this team&apos;s sites go down. Set up
              at least one channel.
            </p>
          )}
          <h3 className="notification-section-title">Channels</h3>
          <ul className="channel-list">
            <ChannelRow
              name="Slack"
              isOn={Boolean(data.slack_webhook_url)}
              detail="Webhook saved"
              offDetail="Post alerts to a Slack channel."
              onEdit={() => setEditing("slack")}
              sendTest={testSlack}
              teamId={team.id}
            />
            <ChannelRow
              name="Microsoft Teams"
              isOn={Boolean(data.microsoft_teams_webhook_url)}
              detail="Webhook saved"
              offDetail="Post alerts to a Teams channel with a workflow webhook."
              onEdit={() => setEditing("teams")}
              sendTest={testTeams}
              teamId={team.id}
            />
            <ChannelRow
              name="Email"
              isOn={isEmailConfigured(data)}
              detail={`${data.smtp_host ?? ""} to ${data.smtp_to_email ?? ""}`}
              offDetail="Send alerts through your own SMTP server."
              onEdit={() => setEditing("email")}
              sendTest={testEmail}
              teamId={team.id}
            />
          </ul>
          <NotificationEventToggles teamId={team.id} settings={data} />

          <Modal
            isOpen={editing === "slack"}
            onClose={closeDialog}
            title={`Slack alerts for ${team.name}`}
          >
            <WebhookForm
              teamId={team.id}
              channel="Slack"
              label="Slack webhook URL"
              placeholder="https://hooks.slack.com/services/..."
              savedWebhookUrl={data.slack_webhook_url ?? ""}
              buildPayload={(url) => ({ slack_webhook_url: url })}
              onDone={closeDialog}
            />
          </Modal>
          <Modal
            isOpen={editing === "teams"}
            onClose={closeDialog}
            title={`Microsoft Teams alerts for ${team.name}`}
          >
            <WebhookForm
              teamId={team.id}
              channel="Microsoft Teams"
              label="Microsoft Teams webhook URL"
              placeholder="https://prod-00.westus.logic.azure.com/workflows/..."
              savedWebhookUrl={data.microsoft_teams_webhook_url ?? ""}
              buildPayload={(url) => ({ microsoft_teams_webhook_url: url })}
              onDone={closeDialog}
            />
          </Modal>
          <Modal
            isOpen={editing === "email"}
            onClose={closeDialog}
            title={`Email alerts for ${team.name}`}
          >
            <SmtpForm teamId={team.id} settings={data} onDone={closeDialog} />
          </Modal>
        </>
      ) : null}
    </section>
  );
}

function ChannelRow({
  name,
  isOn,
  detail,
  offDetail,
  onEdit,
  sendTest,
  teamId,
}: {
  name: string;
  isOn: boolean;
  detail: string;
  offDetail: string;
  onEdit: () => void;
  sendTest: TestNotificationMutation;
  teamId: number;
}) {
  return (
    <li>
      <div className="channel-info">
        <span className="channel-name">
          <span>{name}</span>
          <span className={isOn ? "pill pill-success" : "pill pill-neutral"}>
            {isOn ? "On" : "Off"}
          </span>
        </span>
        <span className="muted-text">{isOn ? detail : offDetail}</span>
        {sendTest.isError && <ErrorMessage error={sendTest.error} />}
        {sendTest.isSuccess && (
          <span className="form-success-text">Test message sent.</span>
        )}
      </div>
      <div className="channel-actions">
        {isOn && (
          <button
            type="button"
            className="button-secondary-action"
            disabled={sendTest.isPending}
            onClick={() => sendTest.mutate(teamId)}
          >
            {sendTest.isPending ? "Sending..." : "Send test"}
          </button>
        )}
        <button
          type="button"
          className={isOn ? "button-secondary-action" : "button-primary-action"}
          aria-haspopup="dialog"
          aria-label={`${isOn ? "Edit" : "Set up"} ${name}`}
          onClick={onEdit}
        >
          {isOn ? "Edit" : "Set up"}
        </button>
      </div>
    </li>
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
    <fieldset className="notification-events">
      <legend className="notification-section-title">Alert when</legend>
      {notificationEvents.map(({ key, label, payload }) => (
        <label key={key} className="switch">
          <input
            type="checkbox"
            role="switch"
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
      {updateEvents.isError && <ErrorMessage error={updateEvents.error} />}
    </fieldset>
  );
}

function DialogActions({
  onTurnOff,
  turnOffLabel,
  onCancel,
  isSaving,
}: {
  onTurnOff?: () => void;
  turnOffLabel: string;
  onCancel: () => void;
  isSaving: boolean;
}) {
  return (
    <div className="dialog-actions">
      {onTurnOff && (
        <button
          type="button"
          className="button-secondary-action button-danger-outline"
          disabled={isSaving}
          onClick={onTurnOff}
        >
          {turnOffLabel}
        </button>
      )}
      <span style={{ flex: "1 1 0" }} />
      <button
        type="button"
        className="button-secondary-action"
        onClick={onCancel}
      >
        Cancel
      </button>
      <button
        type="submit"
        className="button-primary-action"
        disabled={isSaving}
      >
        {isSaving ? "Saving..." : "Save"}
      </button>
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
  onDone,
}: {
  teamId: number;
  channel: string;
  label: string;
  placeholder: string;
  savedWebhookUrl: string;
  buildPayload: (url: string) => UpdateTeamNotificationsRequest;
  onDone: () => void;
}) {
  const updateWebhook = useUpdateTeamNotifications();
  const [webhookUrl, setWebhookUrl] = useState(savedWebhookUrl);
  const save = (url: string) =>
    updateWebhook.mutate(
      { teamId, payload: buildPayload(url) },
      { onSuccess: onDone },
    );

  return (
    <form
      className="form-column"
      onSubmit={(event) => {
        event.preventDefault();
        save(webhookUrl);
      }}
    >
      <label className="field-label">
        <span className="field-label-text">{label}</span>
        <SecretInput
          value={webhookUrl}
          onChange={(event) => setWebhookUrl(event.target.value)}
          placeholder={placeholder}
          required
          style={{ width: "100%" }}
        />
      </label>
      {updateWebhook.isError && <ErrorMessage error={updateWebhook.error} />}
      <DialogActions
        onTurnOff={savedWebhookUrl ? () => save("") : undefined}
        turnOffLabel={`Turn off ${channel}`}
        onCancel={onDone}
        isSaving={updateWebhook.isPending}
      />
    </form>
  );
}

function listMissingFields(fields: string[]) {
  if (fields.length === 1) {
    return fields[0];
  }
  return `${fields.slice(0, -1).join(", ")} and ${fields[fields.length - 1]}`;
}

function SmtpForm({
  teamId,
  settings,
  onDone,
}: {
  teamId: number;
  settings: TeamNotifications;
  onDone: () => void;
}) {
  const updateSmtp = useUpdateTeamNotifications();
  const [host, setHost] = useState(settings.smtp_host ?? "");
  const [port, setPort] = useState(settings.smtp_port?.toString() ?? "");
  const [tlsMode, setTlsMode] = useState(settings.smtp_tls_mode);
  const [smtpAuth, setSmtpAuth] = useState(settings.smtp_auth);
  const [username, setUsername] = useState(settings.smtp_username ?? "");
  const [password, setPassword] = useState("");
  const [fromEmail, setFromEmail] = useState(settings.smtp_from_email ?? "");
  const [toEmail, setToEmail] = useState(settings.smtp_to_email ?? "");
  const [missingFields, setMissingFields] = useState<string[]>([]);

  const findMissingFields = () => {
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
        updateSmtp.mutate({ teamId, payload }, { onSuccess: onDone });
      }}
    >
      <h3 className="notification-section-title">Mail server</h3>
      <label className="field-label">
        <span className="field-label-text">SMTP host</span>
        <FormInput
          value={host}
          onChange={(event) => setHost(event.target.value)}
          placeholder="smtp.waghorn.tech"
        />
      </label>
      <div className="field-row">
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
        <label className="field-label">
          <span className="field-label-text">
            Port <span className="muted-text">(optional)</span>
          </span>
          <FormInput
            type="number"
            min={1}
            max={65535}
            value={port}
            onChange={(event) => setPort(event.target.value)}
            placeholder="Default for the TLS mode"
          />
        </label>
      </div>
      <p className="muted-text" style={{ margin: 0, fontSize: "14px" }}>
        Without a port, 465 is used for TLS, 587 for STARTTLS and 25 with TLS
        off.
      </p>
      <label className="switch">
        <input
          type="checkbox"
          role="switch"
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

      <h3 className="notification-section-title">Addresses</h3>
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

      {missingFields.length > 0 && (
        <p className="error-box">
          Fill in the {listMissingFields(missingFields)} to enable email
          notifications.
        </p>
      )}
      {updateSmtp.isError && <ErrorMessage error={updateSmtp.error} />}
      <DialogActions
        onTurnOff={
          isEmailConfigured(settings)
            ? () =>
                updateSmtp.mutate(
                  { teamId, payload: { smtp_host: "" } },
                  { onSuccess: onDone },
                )
            : undefined
        }
        turnOffLabel="Turn off email"
        onCancel={onDone}
        isSaving={updateSmtp.isPending}
      />
    </form>
  );
}
