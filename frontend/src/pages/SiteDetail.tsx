import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { useOutages, useSite, useUpdateSite } from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { useAuth } from "../hooks/useAuth";
import { SiteForm } from "../components/sites/SiteForm";
import { UptimeHistory } from "../components/sites/UptimeHistory";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { Modal } from "../components/ui/Modal";
import { StatusBadge } from "../components/ui/StatusBadge";
import { ExternalLink } from "../components/ui/ExternalLink";
import { Notice } from "../components/ui/Notice";
import { CertBadge } from "../components/ui/CertBadge";
import { summariseCertificate } from "../lib/certificate";
import { describeSiteStatus } from "../lib/siteStatus";
import { describeUptimeCoverage, formatUptime } from "../lib/uptime";
import { describeCanaryWarning } from "../lib/canary";
import type { components } from "../generated/schema";
import { usePageTitle } from "../hooks/usePageTitle";

type OutageResponse = components["schemas"]["OutageResponse"];
type CanaryWarning = components["schemas"]["CanaryWarning"];

const LAST_PROBE_REFRESH_MS = 1_000;

function formatOutageDuration(startedAt: string, endedAt: string): string {
  const totalSeconds = Math.max(
    0,
    Math.round(
      (new Date(endedAt).getTime() - new Date(startedAt).getTime()) / 1000,
    ),
  );
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
}

function formatTimeAgo(timestamp: string): string {
  const seconds = Math.max(
    0,
    Math.round((Date.now() - new Date(timestamp).getTime()) / 1000),
  );
  if (seconds < 5) return "Just now";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
}

function LastProbe({
  lastCheckedAt,
}: {
  lastCheckedAt: string | null | undefined;
}) {
  const [, refresh] = useState(0);

  useEffect(() => {
    const id = setInterval(() => refresh((n) => n + 1), LAST_PROBE_REFRESH_MS);
    return () => clearInterval(id);
  }, []);

  if (!lastCheckedAt) return <>Never</>;
  return (
    <span title={new Date(lastCheckedAt).toLocaleString()}>
      {formatTimeAgo(lastCheckedAt)}
    </span>
  );
}

const CANARY_NOTICE_TONES: Record<CanaryWarning, "danger" | "warning"> = {
  inactive: "danger",
  shares_host: "warning",
};

function CanaryNotice({
  warning,
  since,
  isAdmin,
}: {
  warning: CanaryWarning;
  since: string | null | undefined;
  isAdmin: boolean;
}) {
  const { explanation, action } = describeCanaryWarning(
    warning,
    since,
    isAdmin,
  );
  return isAdmin ? (
    <Notice
      tone={CANARY_NOTICE_TONES[warning]}
      action={
        <Link to="/admin/canary" className="notice-action">
          {action}
        </Link>
      }
    >
      {explanation}
    </Notice>
  ) : (
    <Notice tone={CANARY_NOTICE_TONES[warning]}>
      {explanation} {action}
    </Notice>
  );
}

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
  usePageTitle(site?.name);
  const updateSite = useUpdateSite();
  const [isEditing, setIsEditing] = useState(false);
  const { data: outagesData, isLoading: outagesLoading } = useOutages(
    siteId,
    page,
  );

  const totalPages = outagesData
    ? Math.ceil(outagesData.total / outagesData.per_page)
    : 0;

  const closeEdit = () => {
    updateSite.reset();
    setIsEditing(false);
  };

  if (siteLoading) return <LoadingSpinner />;
  if (siteError) return <ErrorMessage error={siteError} />;
  if (!site) return <ErrorMessage error={new Error("Site not found")} />;

  const certificate = summariseCertificate(
    site.cert_status,
    site.cert_expires_at,
  );

  return (
    <div className="page-wrapper">
      <Link to="/" className="back-link">
        &larr; Back to Dashboard
      </Link>

      <header className="site-header">
        <div className="site-header-text">
          <h1 className="page-title">{site.name}</h1>
          <p className="site-header-meta">
            <ExternalLink href={site.url}>{site.url}</ExternalLink>
            <span aria-hidden="true">&middot;</span>
            <span>
              Monitoring since {new Date(site.created_at).toLocaleString()}
            </span>
          </p>
        </div>
        <button
          type="button"
          className="button-primary-action"
          aria-haspopup="dialog"
          onClick={() => setIsEditing(true)}
        >
          Edit
        </button>
      </header>

      <Modal isOpen={isEditing} onClose={closeEdit} title="Edit site">
        <SiteForm
          key={`${site.id}-${site.name}-${site.url}`}
          mode="edit"
          initialData={site}
          role={role}
          teams={teams}
          onSubmit={(payload) =>
            updateSite.mutate(
              { id: siteId, site: payload },
              { onSuccess: () => setIsEditing(false) },
            )
          }
          onCancel={closeEdit}
          isLoading={updateSite.isPending}
        />
        {updateSite.isError && <ErrorMessage error={updateSite.error} />}
      </Modal>

      {site.canary_warning && (
        <CanaryNotice
          warning={site.canary_warning}
          since={site.canary_warning_since}
          isAdmin={role === "admin"}
        />
      )}
      {site.status === "blocked" && (
        <Notice tone="neutral">{describeSiteStatus("blocked")}.</Notice>
      )}

      <dl className="site-tiles">
        <div className="site-tile">
          <dt>Status</dt>
          <dd className="site-tile-status">
            <StatusBadge status={site.status} />
          </dd>
          {site.status !== "blocked" && (
            <dd className="site-tile-detail">
              {describeSiteStatus(site.status)}
            </dd>
          )}
          <dd className="site-tile-detail">
            Last probe: <LastProbe lastCheckedAt={site.last_checked_at} />
          </dd>
        </div>
        <div className="site-tile">
          <dt>Uptime</dt>
          <dd className="site-tile-value">
            {site.uptime_basis_points == null
              ? "No data"
              : formatUptime(site.uptime_basis_points)}
          </dd>
          <dd className="site-tile-detail">
            {describeUptimeCoverage(site.created_at, site.uptime_window_days)}
          </dd>
        </div>
        <div className="site-tile">
          <dt>Response time</dt>
          <dd className="site-tile-value">
            {site.last_response_time_ms != null
              ? `${site.last_response_time_ms}ms`
              : "-"}
          </dd>
          <dd className="site-tile-detail">Latest probe</dd>
        </div>
        <div className="site-tile">
          <dt>Certificate</dt>
          <dd className="site-tile-status">
            <CertBadge
              status={site.cert_status}
              expiresAt={site.cert_expires_at}
            />
            <span>{certificate.headline}</span>
          </dd>
          <dd className="site-tile-detail">{certificate.detail}</dd>
        </div>
      </dl>

      <UptimeHistory siteId={siteId} />

      <section>
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            alignItems: "center",
            justifyContent: "space-between",
            gap: "8px 16px",
          }}
        >
          <h2>Outage History</h2>
          {outagesData && outagesData.total > 0 && (
            <a
              className="button-secondary-action"
              href={`/api/sites/${siteId}/outages/export`}
              download
            >
              Download all outages (CSV)
            </a>
          )}
        </div>
        {outagesLoading ? (
          <LoadingSpinner />
        ) : outagesData && outagesData.data.length > 0 ? (
          <>
            <table className="data-table">
              <thead>
                <tr className="table-header-row">
                  <th className="table-cell-left">Started</th>
                  <th className="table-cell-left">Ended</th>
                  <th className="table-cell-left">Duration</th>
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
  const duration = outage.ended_at
    ? formatOutageDuration(outage.started_at, outage.ended_at)
    : "-";

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
      <td className="table-cell">{duration}</td>
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
