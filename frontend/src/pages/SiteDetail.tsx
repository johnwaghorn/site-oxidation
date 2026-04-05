import { useParams, Link } from "react-router-dom";
import { useSite, useUpdateSite, useOutages } from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { SiteForm } from "../components/sites/SiteForm";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { StatusBadge } from "../components/ui/StatusBadge";
import type { components } from "../generated/schema";

type OutageResponse = components["schemas"]["OutageResponse"];

export function SiteDetail() {
  const { id } = useParams<{ id: string }>();
  const siteId = Number(id);
  const { page, goToPage } = usePagination();

  const {
    data: site,
    isLoading: siteLoading,
    error: siteError,
  } = useSite(siteId);
  const updateSite = useUpdateSite();
  const { data: outagesData, isLoading: outagesLoading } = useOutages(
    siteId,
    page,
  );

  const totalPages = outagesData
    ? Math.ceil(outagesData.total / outagesData.per_page)
    : 0;

  if (siteLoading) return <LoadingSpinner />;
  if (siteError) return <ErrorMessage error={siteError} />;
  if (!site) return <ErrorMessage error={new Error("Site not found")} />;

  return (
    <div style={{ maxWidth: "1200px", padding: "24px" }}>
      <Link
        to="/"
        style={{
          color: "#6b7280",
          textDecoration: "none",
          marginBottom: "16px",
          display: "inline-block",
        }}
      >
        &larr; Back to Dashboard
      </Link>

      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "12px",
          marginBottom: "24px",
        }}
      >
        <h1 style={{ margin: 0 }}>{site.name}</h1>
        <StatusBadge status={site.status} />
      </div>

      <section style={{ marginBottom: "32px" }}>
        <h2>Edit Site</h2>
        <SiteForm
          key={`${site.id}-${site.name}-${site.url}`}
          mode="edit"
          initialData={site}
          onSubmit={(payload) =>
            updateSite.mutate({ id: siteId, site: payload })
          }
          isLoading={updateSite.isPending}
        />
        {updateSite.isError && <ErrorMessage error={updateSite.error} />}
        {updateSite.isSuccess && (
          <p style={{ color: "#059669", margin: "8px 0" }}>
            Site updated successfully
          </p>
        )}
      </section>

      <section>
        <h2>Outage History</h2>
        {outagesLoading ? (
          <LoadingSpinner />
        ) : outagesData && outagesData.data.length > 0 ? (
          <>
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <thead>
                <tr style={{ borderBottom: "2px solid #e5e7eb" }}>
                  <th style={{ padding: "12px 8px", textAlign: "left" }}>
                    Started
                  </th>
                  <th style={{ padding: "12px 8px", textAlign: "left" }}>
                    Ended
                  </th>
                  <th style={{ padding: "12px 8px", textAlign: "center" }}>
                    HTTP Status
                  </th>
                  <th style={{ padding: "12px 8px", textAlign: "left" }}>
                    Error
                  </th>
                </tr>
              </thead>
              <tbody>
                {outagesData.data.map((outage: OutageResponse) => (
                  <OutageRow key={outage.id} outage={outage} />
                ))}
              </tbody>
            </table>
            <Pagination
              page={page}
              totalPages={totalPages}
              onPageChange={goToPage}
            />
          </>
        ) : (
          <p style={{ color: "#6b7280" }}>No outages recorded</p>
        )}
      </section>
    </div>
  );
}

function OutageRow({ outage }: { outage: OutageResponse }) {
  const started = new Date(outage.started_at).toLocaleString();
  const ended = outage.ended_at
    ? new Date(outage.ended_at).toLocaleString()
    : "Ongoing";

  return (
    <tr style={{ borderBottom: "1px solid #e5e7eb" }}>
      <td style={{ padding: "12px 8px" }}>{started}</td>
      <td
        style={{
          padding: "12px 8px",
          color: outage.ended_at ? undefined : "#dc2626",
        }}
      >
        {ended}
      </td>
      <td
        style={{
          padding: "12px 8px",
          textAlign: "center",
          fontFamily: "monospace",
        }}
      >
        {outage.http_status ?? "-"}
      </td>
      <td style={{ padding: "12px 8px", color: "#6b7280", fontSize: "14px" }}>
        {outage.error_message ?? "-"}
      </td>
    </tr>
  );
}
