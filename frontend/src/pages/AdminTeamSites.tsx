import { useEffect, useState } from "react";
import { Link, useOutletContext } from "react-router-dom";
import { useAdminTeamSites, useUnassignTeamSite } from "../hooks/useAdmin";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { usePagination } from "../hooks/usePagination";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Pagination } from "../components/ui/Pagination";
import { SearchInput } from "../components/ui/SearchInput";
import { StatusBadge } from "../components/ui/StatusBadge";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import type { components } from "../generated/schema";

type SiteResponse = components["schemas"]["SiteResponse"];
type TeamResponse = components["schemas"]["TeamResponse"];

export function AdminTeamSites() {
  const team = useOutletContext<TeamResponse>();
  const teamId = team.id;
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const {
    data: sites,
    isLoading: sitesLoading,
    error: sitesError,
  } = useAdminTeamSites(teamId, page, perPage, debouncedSearch || undefined);
  const unassignSite = useUnassignTeamSite();
  const [siteToUnassign, setSiteToUnassign] = useState<SiteResponse | null>(
    null,
  );

  useEffect(() => {
    if (sites && sites.data.length === 0 && sites.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [sites, page, goToPage]);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  const totalPages = sites ? Math.ceil(sites.total / sites.per_page) : 0;
  const hasNoSites =
    sites != null && !sitesError && sites.total === 0 && !debouncedSearch;

  return (
    <>
      <p className="muted-text">
        Everyone in this team can see these sites. To move a site to another
        team, open it and choose Edit.
      </p>
      {unassignSite.isError && <ErrorMessage error={unassignSite.error} />}
      {hasNoSites ? (
        <p className="muted-text">
          No sites are assigned to this team yet. Assign one from the{" "}
          <Link to="/">Sites</Link> page.
        </p>
      ) : (
        <>
          <TableCard
            toolbar={
              <SearchInput
                value={searchInput}
                onChange={setSearchInput}
                placeholder="Search sites..."
              />
            }
          >
            {sitesLoading ? (
              <LoadingSpinner />
            ) : sitesError ? (
              <ErrorMessage error={sitesError} />
            ) : sites && sites.data.length > 0 ? (
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
            ) : (
              <TableCardEmpty>No sites match your search.</TableCardEmpty>
            )}
          </TableCard>
          {sites && sites.data.length > 0 && (
            <Pagination
              page={page}
              totalPages={totalPages}
              onPageChange={goToPage}
            />
          )}
        </>
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
    </>
  );
}
