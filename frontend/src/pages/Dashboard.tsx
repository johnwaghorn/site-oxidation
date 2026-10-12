import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  useSites,
  useCreateSite,
  useDeleteSite,
  useUnassignedSiteCount,
} from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { SiteList } from "../components/sites/SiteList";
import { SiteForm } from "../components/sites/SiteForm";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { SearchInput } from "../components/ui/SearchInput";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import { FormToggleButton } from "../components/ui/FormToggleButton";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import type { components } from "../generated/schema";

type SiteResponse = components["schemas"]["SiteResponse"];
type SiteStatusFilter = components["schemas"]["SiteStatusFilter"];
type UserTeam = components["schemas"]["UserTeam"];
type UserRole = components["schemas"]["UserRole"];

const STATUS_FILTERS: { label: string; value: SiteStatusFilter | null }[] = [
  { label: "All", value: null },
  { label: "Up", value: "up" },
  { label: "Down", value: "down" },
];

interface DashboardProps {
  role: UserRole | null;
  teams: UserTeam[];
}

export function Dashboard({ role, teams }: DashboardProps) {
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const [statusFilter, setStatusFilter] = useState<SiteStatusFilter | null>(
    null,
  );
  const { data, isLoading, error } = useSites({
    page,
    perPage,
    search: debouncedSearch || undefined,
    status: statusFilter ?? undefined,
  });
  const { data: orphanedCount = 0 } = useUnassignedSiteCount(role === "admin");
  const createSite = useCreateSite();
  const deleteSite = useDeleteSite();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [siteToDelete, setSiteToDelete] = useState<SiteResponse | null>(null);

  const isFiltered = Boolean(debouncedSearch || statusFilter);
  const totalPages = data ? Math.ceil(data.total / data.per_page) : 0;
  const hasNoSites = data != null && !error && data.total === 0 && !isFiltered;
  const showGetStarted = role === "admin" && hasNoSites;

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, statusFilter, resetPage]);

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Sites</h1>

      <div
        style={{
          display: "flex",
          justifyContent: "flex-end",
          marginBottom: "20px",
        }}
      >
        <FormToggleButton
          isOpen={showCreateForm}
          openLabel="Add Site"
          onClick={() => {
            createSite.reset();
            setShowCreateForm(!showCreateForm);
          }}
        />
      </div>

      {showCreateForm && (
        <div>
          <SiteForm
            onSubmit={(site) =>
              createSite.mutate(site, {
                onSuccess: () => setShowCreateForm(false),
              })
            }
            onCancel={() => {
              createSite.reset();
              setShowCreateForm(false);
            }}
            isLoading={createSite.isPending}
            role={role}
            teams={teams}
          />
          {createSite.isError && <ErrorMessage error={createSite.error} />}
        </div>
      )}

      {hasNoSites ? (
        showGetStarted ? (
          !showCreateForm && (
            <GetStartedNudge onAddSite={() => setShowCreateForm(true)} />
          )
        ) : (
          <SiteList sites={[]} />
        )
      ) : (
        <>
          {role === "admin" && orphanedCount > 0 && (
            <p className="muted-text">
              {orphanedCount} site{orphanedCount === 1 ? "" : "s"} with no team
              assigned
            </p>
          )}
          <TableCard
            toolbar={
              <>
                <SearchInput
                  value={searchInput}
                  onChange={setSearchInput}
                  placeholder="Search sites..."
                />
                <div
                  className="segmented"
                  role="group"
                  aria-label="Filter by status"
                >
                  {STATUS_FILTERS.map((f) => (
                    <button
                      key={f.label}
                      type="button"
                      className="segmented-button"
                      aria-pressed={statusFilter === f.value}
                      onClick={() => setStatusFilter(f.value)}
                    >
                      {f.label}
                    </button>
                  ))}
                </div>
              </>
            }
          >
            {isLoading ? (
              <LoadingSpinner />
            ) : error ? (
              <ErrorMessage error={error} />
            ) : data && data.total === 0 ? (
              <TableCardEmpty>No sites match these filters.</TableCardEmpty>
            ) : data ? (
              <SiteList
                sites={data.data}
                onDelete={(site) => setSiteToDelete(site)}
              />
            ) : null}
          </TableCard>
          {data && (
            <Pagination
              page={data.page}
              totalPages={totalPages}
              onPageChange={goToPage}
            />
          )}
        </>
      )}

      <ConfirmDialog
        isOpen={siteToDelete !== null}
        onClose={() => setSiteToDelete(null)}
        onConfirm={() => siteToDelete && deleteSite.mutate(siteToDelete.id)}
        title="Delete Site"
        message={`Are you sure you want to delete "${siteToDelete?.name}"? This will also delete all outage history.`}
        confirmText="Delete"
        isDestructive
      />
    </div>
  );
}

function GetStartedNudge({ onAddSite }: { onAddSite: () => void }) {
  return (
    <div
      className="card-hero"
      style={{ boxSizing: "border-box", maxWidth: "720px", marginTop: "8px" }}
    >
      <p className="muted-text eyebrow">Welcome</p>
      <h2>Start monitoring your first site</h2>
      <p
        className="muted-text"
        style={{ maxWidth: "560px", margin: "0 0 22px 0" }}
      >
        Add a URL to begin tracking uptime and response history. Teams are
        optional for admins, so you can organise access now or come back to it
        later via the Admin Panel.
      </p>
      <div style={{ display: "flex", flexWrap: "wrap", gap: "12px" }}>
        <button
          type="button"
          className="button-primary-action"
          onClick={onAddSite}
          style={{
            padding: "10px 16px",
          }}
        >
          Add your first site
        </button>
        <Link
          to="/admin/teams"
          style={{
            display: "inline-flex",
            alignItems: "center",
            padding: "10px 16px",
            border: "1px solid var(--color-border)",
            borderRadius: "8px",
            color: "var(--color-text)",
            fontWeight: 500,
          }}
        >
          Set up teams
        </Link>
      </div>
    </div>
  );
}
