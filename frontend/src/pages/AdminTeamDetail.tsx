import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  useAdminTeam,
  useAdminTeamSites,
  useUnassignTeamSite,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Pagination } from "../components/ui/Pagination";
import { StatusBadge } from "../components/ui/StatusBadge";
import { Truncate } from "../components/ui/Truncate";
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
    <div className="page-wrapper">
      <Link to="/admin/teams" className="back-link">
        &larr; Back to Teams
      </Link>

      <h1 className="page-title">
        <Truncate text={team.name} maxWidth="90%" />
      </h1>
      <p className="muted-text">
        {team.member_count} active member{team.member_count === 1 ? "" : "s"}{" "}
        and {team.site_count} assigned site{team.site_count === 1 ? "" : "s"}
      </p>

      <h2>Assigned Sites</h2>
      <p className="muted-text">
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
          <table className="data-table">
            <thead>
              <tr className="table-header-row">
                <th className="table-cell-left">Name</th>
                <th className="table-cell-left">URL</th>
                <th className="table-cell-left">Status</th>
                <th className="table-cell-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {sites.data.map((site) => (
                <tr key={site.id} className="table-row">
                  <td className="table-cell">
                    <Link to={`/sites/${site.id}`}>{site.name}</Link>
                  </td>
                  <td className="table-cell">
                    <a
                      href={site.url}
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      {site.url}
                    </a>
                  </td>
                  <td className="table-cell">
                    <StatusBadge status={site.status} />
                  </td>
                  <td className="table-cell-right">
                    <button
                      className="button-table-action button-table-danger compact-input"
                      onClick={() => setSiteToUnassign(site)}
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
        <p className="muted-text">No sites are assigned to this team.</p>
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
