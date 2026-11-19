import { useId, useState, type FormEvent } from "react";
import { FormInput, FormSelect } from "../ui/FormControls";
import type { components } from "../../generated/schema";

type SitePayload = components["schemas"]["SitePayload"];
type UserTeam = components["schemas"]["UserTeam"];
type UserRole = components["schemas"]["UserRole"];

interface SiteFormProps {
  onSubmit: (site: SitePayload) => void;
  onCancel?: () => void;
  isLoading?: boolean;
  mode?: "create" | "edit";
  initialData?: SitePayload;
  role: UserRole | null;
  teams: UserTeam[];
}

export function SiteForm({
  onSubmit,
  onCancel,
  isLoading,
  mode = "create",
  initialData,
  role,
  teams,
}: SiteFormProps) {
  const [name, setName] = useState(initialData?.name ?? "");
  const [url, setUrl] = useState(initialData?.url ?? "");
  const [expectedStatus, setExpectedStatus] = useState(
    initialData?.expected_status ?? 200,
  );
  const [expectedText, setExpectedText] = useState(
    initialData?.expected_text ?? "",
  );
  const [probeInterval, setProbeInterval] = useState(
    initialData?.probe_interval_seconds ?? 60,
  );
  const [teamId, setTeamId] = useState<number | null>(
    initialData?.team_id ?? null,
  );
  const [tlsAllowUntrusted, setTlsAllowUntrusted] = useState(
    initialData?.tls_allow_untrusted ?? false,
  );

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit({
      name,
      url,
      expected_status: expectedStatus,
      expected_text: expectedText || null,
      probe_interval_seconds: probeInterval,
      team_id: teamId,
      tls_allow_untrusted: tlsAllowUntrusted,
    });
    if (mode === "create") {
      setName("");
      setUrl("");
      setExpectedStatus(200);
      setExpectedText("");
      setTeamId(null);
      setTlsAllowUntrusted(false);
    }
  };

  const hintId = useId();
  const isEdit = mode === "edit";
  const isTeamRequired = role !== "admin";

  return (
    <form onSubmit={handleSubmit} className="form-column">
      <label className="field-label">
        <span className="field-label-text">Site name</span>
        <FormInput
          type="text"
          placeholder="Site name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          minLength={1}
          maxLength={100}
        />
      </label>
      <div className="field-label">
        <label className="field-label">
          <span className="field-label-text">URL</span>
          <FormInput
            type="url"
            placeholder="https://waghorn.tech"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            required
            aria-describedby={`${hintId}-url`}
          />
        </label>
        <p id={`${hintId}-url`} className="field-hint">
          Full URL to monitor, including https://. The cert check only runs for
          https URLs.
        </p>
      </div>
      {teams.length > 0 && (
        <div className="field-label">
          <label className="field-label">
            <span className="field-label-text">Team</span>
            <FormSelect
              value={teamId ?? ""}
              onChange={(e) =>
                setTeamId(e.target.value ? Number(e.target.value) : null)
              }
              required={isTeamRequired}
              aria-describedby={`${hintId}-team`}
            >
              <option value="">
                {isTeamRequired ? "Select team" : "No team"}
              </option>
              {teams.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </FormSelect>
          </label>
          <p id={`${hintId}-team`} className="field-hint">
            Team that owns this site. Only its members (and admins) can see it.
          </p>
        </div>
      )}
      <div className="field-label">
        <label className="field-label">
          <span className="field-label-text">Expected status</span>
          <FormInput
            type="number"
            placeholder="Expected status code"
            value={expectedStatus}
            onChange={(e) => setExpectedStatus(Number(e.target.value))}
            min={100}
            max={599}
            aria-describedby={`${hintId}-status`}
          />
        </label>
        <p id={`${hintId}-status`} className="field-hint">
          The site counts as up only when it returns this HTTP status code.
        </p>
      </div>
      <label className="field-label">
        <span className="field-label-text">Check every</span>
        <FormSelect
          value={probeInterval}
          onChange={(e) => setProbeInterval(Number(e.target.value))}
        >
          <option value={60}>1 minute</option>
          <option value={300}>5 minutes</option>
          <option value={600}>10 minutes</option>
          <option value={1800}>30 minutes</option>
          <option value={3600}>1 hour</option>
        </FormSelect>
      </label>
      <div className="field-label">
        <label className="field-label">
          <span className="field-label-text">
            Expected text <span className="muted-text">(optional)</span>
          </span>
          <FormInput
            type="text"
            placeholder="Expected text (optional)"
            value={expectedText}
            onChange={(e) => setExpectedText(e.target.value)}
            aria-describedby={`${hintId}-text`}
          />
        </label>
        <p id={`${hintId}-text`} className="field-hint">
          Text that must appear in the response body for the site to count as
          up.
        </p>
      </div>
      <div className="field-label">
        <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
          <input
            type="checkbox"
            checked={tlsAllowUntrusted}
            onChange={(e) => setTlsAllowUntrusted(e.target.checked)}
            aria-describedby={`${hintId}-untrusted`}
          />
          Allow untrusted certificate
        </label>
        <p id={`${hintId}-untrusted`} className="field-hint">
          Skip TLS trust and hostname verification for this site. Use for
          internal boxes with untrusted certificates.
        </p>
      </div>
      <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        {onCancel && (
          <button
            type="button"
            className="button-secondary-action"
            onClick={onCancel}
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          className="button-primary-action"
          disabled={isLoading}
        >
          {isLoading
            ? isEdit
              ? "Saving..."
              : "Adding..."
            : isEdit
              ? "Save"
              : "Add Site"}
        </button>
      </div>
    </form>
  );
}
