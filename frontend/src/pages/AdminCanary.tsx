import { useState, type FormEvent } from "react";
import type { components } from "../generated/schema";
import {
  useAdminCanary,
  useTestAdminCanary,
  useUpdateAdminCanary,
} from "../hooks/useAdminCanary";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ExternalLink } from "../components/ui/ExternalLink";
import { FormInput } from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Modal } from "../components/ui/Modal";

type CanaryState = components["schemas"]["CanaryState"];
type CanarySettings = components["schemas"]["CanarySettingsResponse"];

const statePresentation: Record<CanaryState, { label: string; title: string }> =
  {
    disabled: {
      label: "Disabled",
      title:
        "Scheduled checks are off. Every failed probe is treated as a real outage, even if this server lost its connection.",
    },
    unknown: {
      label: "Not checked",
      title:
        "The canary is on but hasn't run yet. Use Check now to run it straight away.",
    },
    healthy: {
      label: "Healthy",
      title: "The last check reached the canary URL successfully.",
    },
    degraded: {
      label: "Degraded",
      title:
        "The last check failed to reach the canary URL. Ambiguous site failures are suppressed until connectivity recovers.",
    },
  };

function formatDate(value: string | null | undefined) {
  return value ? new Date(value).toLocaleString() : "Never";
}

export function AdminCanary() {
  const { data: settings, isLoading, error } = useAdminCanary();
  if (isLoading) {
    return (
      <div className="page-wrapper">
        <LoadingSpinner />
      </div>
    );
  }
  if (error || !settings) {
    return (
      <div className="page-wrapper">
        <h1 className="page-title">Canary</h1>
        <ErrorMessage
          error={error ?? new Error("Settings were not returned")}
        />
      </div>
    );
  }

  return <CanaryStatusPage settings={settings} />;
}

function CanaryStatusPage({ settings }: { settings: CanarySettings }) {
  const toggleChecks = useUpdateAdminCanary();
  const testCanary = useTestAdminCanary();
  const [isEditing, setIsEditing] = useState(false);

  const presentation = statePresentation[settings.state];
  const isLive = settings.state === "healthy";
  const hasUrl = Boolean(settings.url);

  const manualTestResult = testCanary.data;
  const manualTestFailed = manualTestResult?.last_error != null;

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Canary</h1>
      <p className="page-subtitle canary-lede">
        A lightweight check against a URL you control. When it can't connect,
        probe failures are treated as a problem with your own connectivity
        rather than a real site outage.
      </p>

      <section
        className={`canary-status form-card is-${settings.state}`}
        aria-label="Canary status"
      >
        <div className="canary-status-top">
          <div className="canary-status-heading">
            <span
              className={`canary-pulse${isLive ? " is-live" : ""}`}
              aria-hidden="true"
            />
            <h2>Probe connectivity</h2>
            <span
              className="badge canary-state-pill"
              title={presentation.title}
            >
              {presentation.label}
            </span>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              role="switch"
              checked={settings.enabled}
              disabled={
                toggleChecks.isPending || (!hasUrl && !settings.enabled)
              }
              onChange={(event) =>
                toggleChecks.mutate({
                  enabled: event.target.checked,
                  url: settings.url,
                  timeout_secs: settings.timeout_secs,
                })
              }
            />
            Scheduled checks
          </label>
        </div>
        <p className="canary-status-detail">{presentation.title}</p>
        {settings.last_error && (
          <p className="canary-error">{settings.last_error}</p>
        )}
        {toggleChecks.isError && <ErrorMessage error={toggleChecks.error} />}

        <dl className="canary-status-details">
          <div>
            <dt>Canary URL</dt>
            <dd>
              {settings.url ? (
                <ExternalLink href={settings.url}>{settings.url}</ExternalLink>
              ) : (
                "Not set"
              )}
            </dd>
          </div>
          <div>
            <dt>Timeout</dt>
            <dd>
              {settings.timeout_secs}{" "}
              {settings.timeout_secs === 1 ? "second" : "seconds"}
            </dd>
          </div>
          <div>
            <dt>Last checked</dt>
            <dd>{formatDate(settings.last_checked_at)}</dd>
          </div>
          <div>
            <dt>Last successful</dt>
            <dd>{formatDate(settings.last_succeeded_at)}</dd>
          </div>
        </dl>

        <div className="canary-actions">
          {hasUrl ? (
            <>
              <button
                type="button"
                className="button-primary-action"
                onClick={() => testCanary.mutate()}
                disabled={testCanary.isPending}
              >
                {testCanary.isPending ? "Checking..." : "Check now"}
              </button>
              <button
                type="button"
                className="button-secondary-action"
                aria-haspopup="dialog"
                onClick={() => setIsEditing(true)}
              >
                Edit settings
              </button>
            </>
          ) : (
            <button
              type="button"
              className="button-primary-action"
              aria-haspopup="dialog"
              onClick={() => setIsEditing(true)}
            >
              Set canary
            </button>
          )}
          <span className="canary-hint">
            {hasUrl
              ? "Scheduled checks run on the same schedule as your site probes. Check now works even when they are off."
              : "Add a canary URL to turn on checks."}
          </span>
        </div>
        <div role="status">
          {manualTestResult && !testCanary.isPending && (
            <p
              className={
                manualTestFailed
                  ? "canary-test-result is-error"
                  : "canary-test-result is-ok"
              }
            >
              <strong>
                {manualTestFailed ? "Check failed" : "Check passed"}
              </strong>{" "}
              at {formatDate(manualTestResult.last_checked_at)}
              {manualTestFailed
                ? ` - ${manualTestResult.last_error}`
                : ` - ${manualTestResult.url} responded`}
            </p>
          )}
        </div>
        {testCanary.isError && <ErrorMessage error={testCanary.error} />}
      </section>

      <Modal
        isOpen={isEditing}
        onClose={() => setIsEditing(false)}
        title={hasUrl ? "Canary settings" : "Set canary"}
      >
        <CanarySettingsForm
          settings={settings}
          onDone={() => setIsEditing(false)}
        />
      </Modal>
    </div>
  );
}

function CanarySettingsForm({
  settings,
  onDone,
}: {
  settings: CanarySettings;
  onDone: () => void;
}) {
  const updateSettings = useUpdateAdminCanary();
  const [url, setUrl] = useState(settings.url ?? "");
  const [timeoutSecs, setTimeoutSecs] = useState(settings.timeout_secs);
  const [isConfirmingRemove, setIsConfirmingRemove] = useState(false);

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    updateSettings.mutate(
      {
        enabled: settings.enabled,
        url: url.trim() || null,
        timeout_secs: timeoutSecs,
      },
      { onSuccess: onDone },
    );
  };

  return (
    <>
      <form className="form-column" onSubmit={handleSubmit}>
        <label className="field-label">
          <span className="field-label-text">Canary URL</span>
          <FormInput
            type="url"
            value={url}
            onChange={(event) => setUrl(event.target.value)}
            placeholder="https://waghorn.tech"
            required={settings.enabled}
            minLength={10}
            maxLength={2048}
          />
          <span className="canary-hint">
            Pick something reliable that isn't one of your monitored sites.
          </span>
        </label>

        <label className="field-label canary-timeout-field">
          <span className="field-label-text">Timeout in seconds</span>
          <FormInput
            type="number"
            value={timeoutSecs}
            onChange={(event) => setTimeoutSecs(Number(event.target.value))}
            required
            min={1}
            max={300}
          />
        </label>

        {updateSettings.isError && (
          <ErrorMessage error={updateSettings.error} />
        )}

        <div className="dialog-actions">
          {settings.url && (
            <button
              type="button"
              className="button-secondary-action button-danger-outline"
              disabled={updateSettings.isPending}
              onClick={() => setIsConfirmingRemove(true)}
            >
              Remove canary
            </button>
          )}
          <span style={{ flex: "1 1 0" }} />
          <button
            type="button"
            className="button-secondary-action"
            onClick={onDone}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="button-primary-action"
            disabled={updateSettings.isPending}
          >
            {updateSettings.isPending ? "Saving..." : "Save"}
          </button>
        </div>
      </form>
      <ConfirmDialog
        isOpen={isConfirmingRemove}
        onClose={() => setIsConfirmingRemove(false)}
        onConfirm={() =>
          updateSettings.mutate(
            { enabled: false, url: null, timeout_secs: settings.timeout_secs },
            { onSuccess: onDone },
          )
        }
        title="Remove the canary?"
        message="Without one, a drop in the monitor's own internet connection can be recorded as outages on every site."
        confirmText="Remove canary"
        isDestructive
      />
    </>
  );
}
