import { useState, type FormEvent } from "react";
import { inlineForm, compactInput } from "../../lib/styles";
import type { components } from "../../generated/schema";

type SitePayload = components["schemas"]["SitePayload"];
type UserTeam = components["schemas"]["UserTeam"];
type UserRole = components["schemas"]["UserRole"];

interface SiteFormProps {
  onSubmit: (site: SitePayload) => void;
  isLoading?: boolean;
  mode?: "create" | "edit";
  initialData?: SitePayload;
  role: UserRole | null;
  teams: UserTeam[];
}

export function SiteForm({
  onSubmit,
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

  const isEdit = mode === "edit";
  const isTeamRequired = role !== "admin";

  return (
    <form onSubmit={handleSubmit} style={inlineForm}>
      <input
        type="text"
        placeholder="Site name"
        title="Display name to identify this site on the dashboard."
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
        minLength={1}
        maxLength={100}
        style={{ ...compactInput, flex: 1, minWidth: "140px" }}
      />
      <input
        type="url"
        placeholder="https://waghorn.tech"
        title="Full URL to monitor, including https://. The cert check only runs for https URLs."
        value={url}
        onChange={(e) => setUrl(e.target.value)}
        required
        style={{ ...compactInput, flex: 2, minWidth: "220px" }}
      />
      <input
        type="number"
        placeholder="Expected status code"
        title="HTTP status code the site must return to be considered up (e.g. 200). Any other code marks it down."
        value={expectedStatus}
        onChange={(e) => setExpectedStatus(Number(e.target.value))}
        min={100}
        max={599}
        style={{ ...compactInput, width: "100px" }}
      />
      <input
        type="text"
        placeholder="Expected text (optional)"
        title="Optional text that must appear in the response body for the site to count as up."
        value={expectedText}
        onChange={(e) => setExpectedText(e.target.value)}
        style={{ ...compactInput, flex: "1 1 150px", minWidth: "150px" }}
      />
      <select
        value={probeInterval}
        onChange={(e) => setProbeInterval(Number(e.target.value))}
        title="How often this site is checked for availability."
        style={compactInput}
      >
        <option value={60}>1 minute</option>
        <option value={300}>5 minutes</option>
        <option value={600}>10 minutes</option>
        <option value={1800}>30 minutes</option>
        <option value={3600}>1 hour</option>
      </select>
      {teams.length > 0 && (
        <select
          value={teamId ?? ""}
          onChange={(e) =>
            setTeamId(e.target.value ? Number(e.target.value) : null)
          }
          required={isTeamRequired}
          title="Team that owns this site. Only its members (and admins) can see it."
          style={compactInput}
        >
          <option value="">{isTeamRequired ? "Select team" : "No team"}</option>
          {teams.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
      )}
      <label
        style={{
          display: "flex",
          alignItems: "center",
          gap: "4px",
          fontSize: "14px",
          whiteSpace: "nowrap",
        }}
        title="Skip TLS trust and hostname verification for this site. Use for internal boxes with untrusted certs."
      >
        <input
          type="checkbox"
          checked={tlsAllowUntrusted}
          onChange={(e) => setTlsAllowUntrusted(e.target.checked)}
        />
        Allow untrusted
      </label>
      <button type="submit" disabled={isLoading} style={compactInput}>
        {isLoading
          ? isEdit
            ? "Saving..."
            : "Adding..."
          : isEdit
            ? "Save"
            : "Add Site"}
      </button>
    </form>
  );
}
