import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  useAdminTeam,
  useAdminTeamSites,
  useUnassignTeamSite,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { AdminNav } from "../components/ui/AdminNav";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Pagination } from "../components/ui/Pagination";
import { StatusBadge } from "../components/ui/StatusBadge";
import { Truncate } from "../components/ui/Truncate";
import {
  backLink,
  compactInput,
  mutedText,
  pageTitle,
  pageWrapper,
  table,
  tableCell,
  tableCellLeft,
  tableCellRight,
  tableHeaderRow,
  tableRow,
} from "../lib/styles";
import type { components } from "../generated/schema";

type SiteResponse = components["schemas"]["SiteResponse"];

export function AdminTeamDetail() {
  const { id } = useParams<{ id: string }>();
  const teamId = Number(id);
  const { page, perPage, goToPage } = usePagination();
  const {
    data: team,
    isLoading: teamLoading,
    error: teamError,
  } = useAdminTeam(teamId);
  const {
    data: sites,
    isLoading: sitesLoading,
    error: sitesError,
  } = useAdminTeamSites(teamId, page, perPage);
  const unassignSite = useUnassignTeamSite();
  const [siteToUnassign, setSiteToUnassign] = useState<SiteResponse | null>(
    null,
  );

  useEffect(() => {
    if (sites && sites.data.length === 0 && sites.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [sites, page, goToPage]);

  if (!Number.isInteger(teamId) || teamId <= 0) {
    return <ErrorMessage error={new Error("Team not found")} />;
  }
  if (teamLoading) return <LoadingSpinner />;
  if (teamError) return <ErrorMessage error={teamError} />;
  if (!team) return <ErrorMessage error={new Error("Team not found")} />;

  const totalPages = sites ? Math.ceil(sites.total / sites.per_page) : 0;

  return (
    <div style={pageWrapper}>
      <Link to="/admin/teams" style={backLink}>
        &larr; Back to Teams
      </Link>
      <AdminNav />

      <h1 style={pageTitle}>
        <Truncate text={team.name} maxWidth="90%" />
      </h1>
      <p style={mutedText}>
        {team.member_count} active member{team.member_count === 1 ? "" : "s"}{" "}
        and {team.site_count} assigned site{team.site_count === 1 ? "" : "s"}
      </p>

      <h2>Assigned Sites</h2>
      <p style={mutedText}>
        Open a site to reassign it to another team, or remove its assignment
        here.
      </p>
      {unassignSite.isError && <ErrorMessage error={unassignSite.error} />}
      {sitesLoading ? (
        <LoadingSpinner />
      ) : sitesError ? (
        <ErrorMessage error={sitesError} />
      ) : sites && sites.data.length > 0 ? (
        <>
          <table style={table}>
            <thead>
              <tr style={tableHeaderRow}>
                <th style={tableCellLeft}>Name</th>
                <th style={tableCellLeft}>URL</th>
                <th style={tableCellLeft}>Status</th>
                <th style={tableCellRight}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {sites.data.map((site) => (
                <tr key={site.id} style={tableRow}>
                  <td style={tableCell}>
                    <Link to={`/sites/${site.id}`}>{site.name}</Link>
                  </td>
                  <td style={tableCell}>
                    <a
                      href={site.url}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {site.url}
                    </a>
                  </td>
                  <td style={tableCell}>
                    <StatusBadge status={site.status} />
                  </td>
                  <td style={tableCellRight}>
                    <button
                      className="button-table-action button-table-danger"
                      onClick={() => setSiteToUnassign(site)}
                      style={compactInput}
                    >
                      Remove from team
                    </button>
                  </td>
                </tr>
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
        <p style={mutedText}>No sites are assigned to this team.</p>
      )}

      <ConfirmDialog
        isOpen={siteToUnassign !== null}
        onClose={() => setSiteToUnassign(null)}
        onConfirm={() =>
          siteToUnassign &&
          unassignSite.mutate({ teamId, siteId: siteToUnassign.id })
        }
        title="Remove Site from Team"
        message={`Remove "${siteToUnassign?.name}" from "${team.name}"? The site will remain monitored but will no longer belong to a team.`}
        confirmText="Remove"
        isDestructive
      />
    </div>
  );
}
