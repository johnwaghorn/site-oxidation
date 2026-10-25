import { useRef, useState, type DragEvent } from "react";
import { useNavigate } from "react-router-dom";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { FormSelect } from "../components/ui/FormControls";
import {
  FileCheckIcon,
  FileDownIcon,
  FileUpIcon,
  FileXIcon,
} from "../components/icons";
import { useAuth } from "../hooks/useAuth";
import { useExportSites, useImportSites } from "../hooks/useImportExport";
import type { components } from "../generated/schema";

type SiteImportRequest = components["schemas"]["SiteImportRequest"];
type SiteImportResponse = components["schemas"]["SiteImportResponse"];

type ImportView =
  | { kind: "ready" }
  | { kind: "importing"; fileName: string }
  | {
      kind: "done";
      fileName: string;
      teamName: string;
      result: SiteImportResponse;
    }
  | {
      kind: "rejected";
      fileName: string;
      location: string | null;
      detail: string;
    };

const SERVER_JSON_ERROR_PREFIX =
  "Failed to deserialize the JSON body into the target type: ";
const SERVER_JSON_ERROR_POSITION_SUFFIX = / at line \d+ column \d+$/;
const SITE_FIELD_ERROR = /^sites\[(\d+)\][^:]*: (.*)$/;

function downloadJson(filename: string, data: unknown) {
  const blob = new Blob([JSON.stringify(data, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function isSiteExportFile(value: unknown): value is SiteImportRequest {
  return (
    typeof value === "object" &&
    value !== null &&
    "version" in value &&
    typeof value.version === "number" &&
    "sites" in value &&
    Array.isArray(value.sites)
  );
}

function stripServerJsonErrorWrapping(message: string) {
  return message
    .replace(SERVER_JSON_ERROR_PREFIX, "")
    .replace(SERVER_JSON_ERROR_POSITION_SUFFIX, "");
}

function capitalise(text: string) {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

function splitImportErrorIntoSiteNumberAndDetail(message: string) {
  const text = stripServerJsonErrorWrapping(message);
  const siteFieldError = SITE_FIELD_ERROR.exec(text);
  if (!siteFieldError) return { location: null, detail: capitalise(text) };
  const [, siteIndex, detail] = siteFieldError;
  return {
    location: `Site ${Number(siteIndex) + 1} in the file`,
    detail: capitalise(detail),
  };
}

function isDraggingFiles(event: DragEvent) {
  return event.dataTransfer.types.includes("Files");
}

export function ImportExport() {
  const navigate = useNavigate();
  const { role, teams } = useAuth();
  const exportSites = useExportSites();
  const importSites = useImportSites();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const isTeamRequired = role !== "admin";
  const [teamId, setTeamId] = useState<number | null>(teams[0]?.id ?? null);
  const [dragDepth, setDragDepth] = useState(0);
  const [view, setView] = useState<ImportView>({ kind: "ready" });

  const canImport = !isTeamRequired || teamId !== null;
  const teamName = teams.find((team) => team.id === teamId)?.name ?? "no team";

  async function importFile(file: File) {
    if (!canImport) return;
    let parsed: unknown;
    try {
      parsed = JSON.parse(await file.text());
    } catch {
      setView({
        kind: "rejected",
        fileName: file.name,
        location: null,
        detail:
          "This is not a JSON file. Use a file exported from Site Oxidation.",
      });
      return;
    }
    if (!isSiteExportFile(parsed)) {
      setView({
        kind: "rejected",
        fileName: file.name,
        location: null,
        detail: "This does not look like a Site Oxidation export file.",
      });
      return;
    }
    setView({ kind: "importing", fileName: file.name });
    importSites.mutate(
      { teamId, file: parsed },
      {
        onSuccess: (result) =>
          setView({ kind: "done", fileName: file.name, teamName, result }),
        onError: (error) =>
          setView({
            kind: "rejected",
            fileName: file.name,
            ...splitImportErrorIntoSiteNumberAndDetail(error.message),
          }),
      },
    );
  }

  const addedCount =
    view.kind === "done"
      ? view.result.sites.filter((site) => site.outcome === "added").length
      : 0;
  const skippedCount =
    view.kind === "done" ? view.result.sites.length - addedCount : 0;
  const isDragging = dragDepth > 0;

  return (
    <div
      className={`page-wrapper transfer-page${isDragging ? " is-dragging" : ""}`}
      onDragEnter={(event) => {
        if (!isDraggingFiles(event)) return;
        event.preventDefault();
        setDragDepth((depth) => depth + 1);
      }}
      onDragOver={(event) => {
        if (isDraggingFiles(event)) event.preventDefault();
      }}
      onDragLeave={(event) => {
        if (isDraggingFiles(event))
          setDragDepth((depth) => Math.max(0, depth - 1));
      }}
      onDrop={(event) => {
        if (!isDraggingFiles(event)) return;
        event.preventDefault();
        setDragDepth(0);
        const file = event.dataTransfer.files[0];
        if (file) void importFile(file);
      }}
    >
      <div>
        <h1 className="page-title">Import & Export</h1>
        <p className="muted-text" style={{ margin: 0 }}>
          Bring sites in from a file, or take a copy of yours with you.
        </p>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept="application/json,.json"
        className="visually-hidden"
        tabIndex={-1}
        aria-hidden="true"
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (file) void importFile(file);
        }}
      />

      <div className="transfer-stage" aria-live="polite">
        {isDragging || view.kind === "ready" ? (
          <div className="transfer-split">
            <div className="transfer-import">
              <div className="transfer-file" aria-hidden="true">
                <FileDownIcon />
                IN
              </div>
              <p className="transfer-subheadline">
                {isDragging ? "Let go to import" : "Drop a file to import"}
              </p>
              {isTeamRequired && teams.length === 0 ? (
                <p className="muted-text" style={{ margin: 0 }}>
                  You need to be in a team before you can import sites.
                </p>
              ) : (
                <>
                  <label className="transfer-team">
                    into team
                    <FormSelect
                      value={teamId ?? ""}
                      onChange={(event) =>
                        setTeamId(
                          event.target.value
                            ? Number(event.target.value)
                            : null,
                        )
                      }
                    >
                      {!isTeamRequired && <option value="">No team</option>}
                      {teams.map((team) => (
                        <option key={team.id} value={team.id}>
                          {team.name}
                        </option>
                      ))}
                    </FormSelect>
                  </label>
                  <p className="muted-text" style={{ margin: 0 }}>
                    Anywhere on the page works. Sites already in the team are
                    skipped.{" "}
                    <button
                      type="button"
                      className="button-link"
                      onClick={() => fileInputRef.current?.click()}
                    >
                      Choose a file
                    </button>
                  </p>
                </>
              )}
            </div>
            <div className="transfer-or" aria-hidden="true">
              or
            </div>
            <section className="transfer-export">
              <div className="transfer-file is-outgoing" aria-hidden="true">
                <FileUpIcon />
                OUT
              </div>
              <h2 className="transfer-subheadline">Export your sites</h2>
              <p className="muted-text" style={{ margin: 0 }}>
                Every site you can see, with its check settings, in one file.
              </p>
              <button
                className="button-primary-action"
                disabled={exportSites.isPending}
                onClick={() =>
                  exportSites.mutate(undefined, {
                    onSuccess: (data) =>
                      downloadJson(
                        `site-oxidation-sites-${data.exported_at.slice(0, 10)}.json`,
                        data,
                      ),
                  })
                }
              >
                {exportSites.isPending ? "Exporting..." : "Export sites"}
              </button>
              <ErrorMessage error={exportSites.error} />
            </section>
          </div>
        ) : view.kind === "importing" ? (
          <>
            <div className="transfer-file" aria-hidden="true">
              <FileDownIcon />
              IN
            </div>
            <p className="transfer-headline">Importing {view.fileName}...</p>
          </>
        ) : view.kind === "done" ? (
          <>
            <div className="transfer-file is-done" aria-hidden="true">
              <FileCheckIcon />
              DONE
            </div>
            <p className="muted-text" style={{ margin: 0 }}>
              {view.fileName} → <strong>{view.teamName}</strong>
            </p>
            <div className="transfer-tiles">
              <div className="transfer-tile is-added">
                <div className="transfer-tile-count">{addedCount}</div>
                <div className="transfer-tile-label">
                  {addedCount === 1 ? "site added" : "sites added"}
                </div>
              </div>
              <div className="transfer-tile">
                <div className="transfer-tile-count">{skippedCount}</div>
                <div className="transfer-tile-label">already here</div>
              </div>
            </div>
            {view.result.sites.length > 0 && (
              <details className="transfer-details">
                <summary>
                  Show all {view.result.sites.length}{" "}
                  {view.result.sites.length === 1 ? "site" : "sites"}
                </summary>
                <ul>
                  {view.result.sites.map((site, index) => (
                    <li key={index}>
                      <span>
                        <strong>{site.name}</strong>{" "}
                        <span className="muted-text">{site.url}</span>
                      </span>
                      <span
                        className={`transfer-outcome${site.outcome === "added" ? " is-added" : ""}`}
                      >
                        {site.outcome === "added" ? "Added" : "Already here"}
                      </span>
                    </li>
                  ))}
                </ul>
              </details>
            )}
            <button
              className="button-primary-action"
              onClick={() => void navigate("/")}
            >
              View sites
            </button>
            <p className="muted-text" style={{ margin: 0 }}>
              Drop another file to import again, or{" "}
              <button
                type="button"
                className="button-link"
                onClick={() => fileInputRef.current?.click()}
              >
                choose a file
              </button>
              .
            </p>
          </>
        ) : (
          <>
            <div className="transfer-file is-rejected" aria-hidden="true">
              <FileXIcon />
              REJECTED
            </div>
            <p className="transfer-headline">Nothing was imported</p>
            <p className="muted-text" style={{ margin: 0, maxWidth: "460px" }}>
              {view.fileName} has a problem, so none of its sites were added.
              Your sites are unchanged.
            </p>
            <div className="transfer-problem" role="alert">
              {view.location && <strong>{view.location}</strong>}
              <span>{view.detail}</span>
            </div>
            <p className="muted-text" style={{ margin: 0 }}>
              Fix it and drop the file again, or{" "}
              <button
                type="button"
                className="button-link"
                onClick={() => fileInputRef.current?.click()}
              >
                choose a file
              </button>
              .
            </p>
          </>
        )}
      </div>
    </div>
  );
}
