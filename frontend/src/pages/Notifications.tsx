import { useEffect, useState } from "react";
import { Link, Navigate, useParams } from "react-router-dom";
import type { UseMutationResult } from "@tanstack/react-query";
import { TeamNotificationsTable } from "../components/notifications/TeamNotificationsTable";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import {
  FormInput,
  FormSelect,
  SecretInput,
} from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Modal } from "../components/ui/Modal";
import { Pagination } from "../components/ui/Pagination";
import { SearchInput } from "../components/ui/SearchInput";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import { useAuth } from "../hooks/useAuth";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { usePagination } from "../hooks/usePagination";
import {
  useTeamNotifications,
  useTeamNotificationSummaries,
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
type NotificationEventKey = Extract<
  keyof UpdateTeamNotificationsRequest,
  `notify_${string}`
>;

const SMTP_TLS_MODE_LABELS: Record<SmtpTlsMode, string> = {
  starttls: "STARTTLS",
  tls: "TLS",
  none: "None",
};

const SENDS_ALERTS_FILTERS: {
  label: string;
  sendsAlerts: boolean | null;
}[] = [
  { label: "All", sendsAlerts: null },
  { label: "Alerts on", sendsAlerts: true },
  { label: "No alerts", sendsAlerts: false },
];

function isEmailConfigured(settings: TeamNotifications | undefined) {
  return Boolean(
    settings?.smtp_host && settings.smtp_from_email && settings.smtp_to_email,
  );
}

function isEveryAlertSwitchedOff(settings: TeamNotifications) {
  return (
    !settings.notify_site_down &&
    !settings.notify_site_recovered &&
    !settings.notify_cert_expiring
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
  const hasSeveralTeams = teams.length > 1;
  const selected =
    teams.find((team) => team.id === Number(teamId)) ??
    (teams.length === 1 ? teams[0] : null);

  if (!isLoading && teamId && !selected) {
    return <Navigate to="/notifications" replace />;
  }

  return (
    <div className="page-wrapper">
      {selected && hasSeveralTeams && (
        <Link to="/notifications" className="back-link">
          &larr; Back to Notifications
        </Link>
      )}
      <h1 className="page-title">Notifications</h1>
      <p className="page-subtitle">
        Each team chooses where its alerts go. Every site in a team uses that
        team&apos;s settings.
      </p>

      {isLoading ? (
        <LoadingSpinner />
      ) : teams.length === 0 ? (
        <section className="card-hero">
          <h2>No teams available</h2>
          <p className="muted-text" style={{ maxWidth: "520px", margin: 0 }}>
            Join or create a team before configuring notifications.
          </p>
        </section>
      ) : selected ? (
        <TeamSettings key={selected.id} team={selected} />
      ) : (
        <TeamNotificationsList />
      )}
    </div>
  );
}

function TeamNotificationsList() {
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const [sendsAlertsFilter, setSendsAlertsFilter] = useState<boolean | null>(
    null,
  );
  const { page, perPage, goToPage, correctPage } = usePagination(
    `${debouncedSearch}|${sendsAlertsFilter ?? ""}`,
  );
  const { data, isLoading, error } = useTeamNotificationSummaries({
    page,
    perPage,
    search: debouncedSearch || undefined,
    sendsAlerts: sendsAlertsFilter ?? undefined,
  });
  const totalPages = data ? Math.ceil(data.total / data.per_page) : 0;

  useEffect(() => {
    if (data && data.data.length === 0 && data.total > 0 && page > 1) {
      correctPage(1);
    }
  }, [data, page, correctPage]);

  return (
    <>
      <TableCard
        toolbar={
          <>
            <SearchInput
              value={searchInput}
              onChange={setSearchInput}
              placeholder="Search teams..."
            />
            <div
              className="segmented"
              role="group"
              aria-label="Filter by alerts"
            >
              {SENDS_ALERTS_FILTERS.map((filter) => (
                <button
                  key={filter.label}
                  type="button"
                  className="segmented-button"
                  aria-pressed={sendsAlertsFilter === filter.sendsAlerts}
                  onClick={() => setSendsAlertsFilter(filter.sendsAlerts)}
                >
                  {filter.label}
                </button>
              ))}
            </div>
          </>
        }
      >
        {isLoading ? (
          <LoadingSpinner />
        ) : error ? (
          <ErrorMessage error={error} />
        ) : data && data.total === 0 ? (
          <TableCardEmpty>No teams match these filters.</TableCardEmpty>
        ) : data ? (
          <TeamNotificationsTable summaries={data.data} />
        ) : null}
      </TableCard>
      {data && (
        <Pagination
          page={data.page}
          totalPages={totalPages}
          onPageChange={goToPage}
        />
      )}
    </>
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
    <section className="card" aria-labelledby={`team-${team.id}`}>
      <h2 id={`team-${team.id}`} style={{ margin: "0 0 16px" }}>
        {team.name}
      </h2>
      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : data ? (
        <>
          {countChannelsOn(data) === 0 ? (
            <p className="warning-box">
              Nobody will be alerted when this team&apos;s sites go down. Set up
              at least one channel.
            </p>
          ) : isEveryAlertSwitchedOff(data) ? (
            <p className="warning-box">
              Nobody will be alerted because every alert below is switched off.
              Turn on at least one.
            </p>
          ) : null}
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
          <span className="form-success-text" role="status">
            Test message sent.
          </span>
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
  key: NotificationEventKey;
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
            {Object.entries(SMTP_TLS_MODE_LABELS).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
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
        <p className="error-box" role="alert">
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
                  {
                    teamId,
                    payload: {
                      smtp_host: "",
                      smtp_from_email: "",
                      smtp_to_email: "",
                    },
                  },
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
