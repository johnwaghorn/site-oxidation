import { useState, type FormEvent } from "react";
import type { components } from "../generated/schema";
import {
  useAdminCanary,
  useTestAdminCanary,
  useUpdateAdminCanary,
} from "../hooks/useAdminCanary";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { FormInput } from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";

type CanaryState = components["schemas"]["CanaryState"];
type CanarySettings = components["schemas"]["CanarySettingsResponse"];

const statePresentation: Record<CanaryState, { label: string; title: string }> =
  {
    disabled: {
      label: "Disabled",
      title:
        "Canary checks are turned off. Site will be probed without connectivity based outage suppression.",
    },
    unknown: {
      label: "Not checked",
      title: "Canary is enabled but a check hasn't run yet!",
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

  const configKey = `${settings.enabled}:${settings.timeout_secs}:${settings.url}`;
  return <CanarySettingsPage key={configKey} settings={settings} />;
}

function CanarySettingsPage({ settings }: { settings: CanarySettings }) {
  const updateSettings = useUpdateAdminCanary();
  const testCanary = useTestAdminCanary();
  const [enabled, setEnabled] = useState(settings.enabled);
  const [url, setUrl] = useState(settings.url ?? "");
  const [timeoutSecs, setTimeoutSecs] = useState(settings.timeout_secs);

  const isDirty =
    enabled !== settings.enabled ||
    url !== (settings.url ?? "") ||
    timeoutSecs !== settings.timeout_secs;
  const presentation = statePresentation[settings.state];
  const isLive = settings.state === "healthy";

  const manualTestResult = testCanary.data;
  const manualTestFailed = manualTestResult?.last_error != null;
  const testHint = !url.trim()
    ? "Add a canary URL to test"
    : isDirty
      ? "Save changes before testing"
      : null;

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();
    updateSettings.mutate({
      enabled,
      url: url.trim() || null,
      timeout_secs: timeoutSecs,
    });
  };

  return (
    <div className="page-wrapper canary-page">
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
        <div className="canary-status-heading">
          <span
            className={`canary-pulse${isLive ? " is-live" : ""}`}
            aria-hidden="true"
          />
          <h2>Probe connectivity</h2>
          <span className="badge canary-state-pill" title={presentation.title}>
            {presentation.label}
          </span>
        </div>
        <p className="canary-status-detail">{presentation.title}</p>
        {settings.last_error && (
          <p className="canary-error">{settings.last_error}</p>
        )}
        <dl className="canary-status-details">
          <div>
            <dt>Last checked</dt>
            <dd>{formatDate(settings.last_checked_at)}</dd>
          </div>
          <div>
            <dt>Last successful</dt>
            <dd>{formatDate(settings.last_succeeded_at)}</dd>
          </div>
        </dl>
      </section>

      <section
        className="canary-settings form-card"
        aria-labelledby="canary-settings-title"
      >
        <h2 id="canary-settings-title">Connectivity canary</h2>
        <p className="canary-settings-hint">
          Runs on the same schedule as your site probes. Disabling the canary
          only stops its own scheduled checks. A manual check will still work.
        </p>

        <form className="canary-form" onSubmit={handleSubmit}>
          <label className="canary-toggle">
            <input
              type="checkbox"
              checked={enabled}
              onChange={(event) => setEnabled(event.target.checked)}
            />
            Canary enabled
          </label>

          <label className="field-label">
            <span className="field-label-text">Canary URL</span>
            <div className="canary-url-row">
              <FormInput
                type="url"
                value={url}
                onChange={(event) => setUrl(event.target.value)}
                placeholder="https://waghorn.tech"
                required={enabled}
                minLength={10}
                maxLength={2048}
              />
              <button
                type="button"
                className="button-secondary-action"
                onClick={() => testCanary.mutate()}
                disabled={testCanary.isPending || isDirty || !url.trim()}
              >
                {testCanary.isPending ? "Testing..." : "Test"}
              </button>
            </div>
            <div className="canary-test-status" role="status">
              {testCanary.isPending ? (
                <p className="canary-test-result is-pending">
                  Testing {url.trim()}...
                </p>
              ) : manualTestResult ? (
                <p
                  className={
                    manualTestFailed
                      ? "canary-test-result is-error"
                      : "canary-test-result is-ok"
                  }
                >
                  <strong>
                    {manualTestFailed ? "Test failed" : "Test passed"}
                  </strong>{" "}
                  at {formatDate(manualTestResult.last_checked_at)}
                  {manualTestFailed
                    ? ` - ${manualTestResult.last_error}`
                    : ` - ${manualTestResult.url} responded`}
                </p>
              ) : (
                testHint && <span className="canary-hint">{testHint}</span>
              )}
            </div>
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

          <div className="canary-actions">
            <button
              type="submit"
              className="button-primary-action"
              disabled={!isDirty || updateSettings.isPending}
            >
              {updateSettings.isPending ? "Saving..." : "Save changes"}
            </button>
          </div>
        </form>

        {updateSettings.isError && (
          <ErrorMessage error={updateSettings.error} />
        )}
        {testCanary.isError && <ErrorMessage error={testCanary.error} />}
      </section>
    </div>
  );
}
