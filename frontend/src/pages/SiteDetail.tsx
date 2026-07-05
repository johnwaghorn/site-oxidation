import { useParams, Link } from "react-router-dom";
import { useSite, useUpdateSite, useOutages } from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { useAuth } from "../hooks/useAuth";
import { SiteForm } from "../components/sites/SiteForm";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { StatusBadge } from "../components/ui/StatusBadge";
import { CertBadge } from "../components/ui/CertBadge";
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
    <div className="page-wrapper">
      <Link to="/" className="back-link">
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
        <h1 className="page-title" style={{ margin: 0 }}>
          {site.name}
        </h1>
        <StatusBadge status={site.status} />
        <CertBadge status={site.cert_status} expiresAt={site.cert_expires_at} />
      </div>

      <div
        className="muted-text"
        style={{ marginTop: "-12px", marginBottom: "24px" }}
      >
        <p style={{ margin: "0 0 4px 0" }}>
          Monitoring since: {new Date(site.created_at).toLocaleString()}
        </p>
        {site.cert_expires_at && (
          <p style={{ margin: 0 }}>
            Certificate expiry:{" "}
            {new Date(site.cert_expires_at).toLocaleDateString()}
          </p>
        )}
      </div>

      <section className="page-section">
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
          <p style={{ color: "var(--color-success)", margin: "8px 0" }}>
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
            <table className="data-table">
              <thead>
                <tr className="table-header-row">
                  <th className="table-cell-left">Started</th>
                  <th className="table-cell-left">Ended</th>
                  <th className="table-cell-center">Expected Status</th>
                  <th className="table-cell-center">Actual Status</th>
                  <th className="table-cell-left">Error</th>
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
          <p className="muted-text">No outages recorded</p>
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
    <tr className="table-row">
      <td className="table-cell">{started}</td>
      <td
        className="table-cell"
        style={{
          color: outage.ended_at ? undefined : "var(--color-danger)",
        }}
      >
        {ended}
      </td>
      <td
        className="table-cell-center"
        style={{
          fontFamily: "monospace",
        }}
      >
        {outage.expected_status ?? "-"}
      </td>
      <td
        className="table-cell-center"
        style={{
          fontFamily: "monospace",
        }}
      >
        {outage.http_status ?? "-"}
      </td>
      <td className="table-cell muted-text" style={{ fontSize: "14px" }}>
        {outage.error_message ?? "-"}
      </td>
    </tr>
  );
}
