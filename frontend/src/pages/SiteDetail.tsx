import { useParams, Link } from "react-router-dom";
import { useSite, useUpdateSite, useOutages } from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { useAuth } from "../hooks/useAuth";
import { SiteForm } from "../components/sites/SiteForm";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { StatusBadge } from "../components/ui/StatusBadge";
import {
  pageWrapper,
  pageTitle,
  backLink,
  section,
  mutedText,
  table,
  tableHeaderRow,
  tableRow,
  tableCell,
  tableCellLeft,
  tableCellCenter,
} from "../lib/styles";
import type { components } from "../generated/schema";

type OutageResponse = components["schemas"]["OutageResponse"];

export function SiteDetail() {
  const { id } = useParams<{ id: string }>();
  const siteId = Number(id);
  const { page, goToPage } = usePagination();
  const { role, teams } = useAuth();

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
    <div style={pageWrapper}>
      <Link to="/" style={backLink}>
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
        <h1 style={pageTitle}>{site.name}</h1>
        <StatusBadge status={site.status} />
      </div>

      <section style={section}>
        <h2>Edit Site</h2>
        <SiteForm
          key={`${site.id}-${site.name}-${site.url}`}
          mode="edit"
          initialData={site}
          role={role}
          teams={teams}
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
            <table style={table}>
              <thead>
                <tr style={tableHeaderRow}>
                  <th style={tableCellLeft}>Started</th>
                  <th style={tableCellLeft}>Ended</th>
                  <th style={tableCellCenter}>HTTP Status</th>
                  <th style={tableCellLeft}>Error</th>
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
          <p style={mutedText}>No outages recorded</p>
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
    <tr style={tableRow}>
      <td style={tableCell}>{started}</td>
      <td
        style={{
          ...tableCell,
          color: outage.ended_at ? undefined : "#dc2626",
        }}
      >
        {ended}
      </td>
      <td
        style={{
          ...tableCellCenter,
          fontFamily: "monospace",
        }}
      >
        {outage.http_status ?? "-"}
      </td>
      <td style={{ ...tableCell, ...mutedText, fontSize: "14px" }}>
        {outage.error_message ?? "-"}
      </td>
    </tr>
  );
}
