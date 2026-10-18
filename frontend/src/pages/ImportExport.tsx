import { ErrorMessage } from "../components/ui/ErrorMessage";
import { useExportSites } from "../hooks/useImportExport";

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

export function ImportExport() {
  const exportSites = useExportSites();

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Import & Export</h1>
      <section className="card-hero" style={{ maxWidth: "680px" }}>
        <h2>Export sites</h2>
        <p className="muted-text" style={{ maxWidth: "520px" }}>
          Download the settings of every site you can see as a JSON file, ready
          to import into another Site Oxidation instance. Outage history and
          notification settings are not included.
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
  );
}
