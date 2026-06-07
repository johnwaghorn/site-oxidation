import { useEffect, useState } from "react";
import { useSites, useCreateSite, useDeleteSite } from "../hooks/useSites";
import { usePagination } from "../hooks/usePagination";
import { SiteList } from "../components/sites/SiteList";
import { SiteForm } from "../components/sites/SiteForm";
import { Pagination } from "../components/ui/Pagination";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { UserMenu } from "../components/ui/UserMenu";
import { SearchInput, SearchToolbar } from "../components/ui/SearchInput";
import { FormToggleButton } from "../components/ui/FormToggleButton";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import {
  pageWrapper,
  headerRow,
  pageTitle,
  compactInput,
  mutedText,
} from "../lib/styles";
import type { components } from "../generated/schema";

type SiteResponse = components["schemas"]["SiteResponse"];
type UserTeam = components["schemas"]["UserTeam"];
type UserRole = components["schemas"]["UserRole"];

interface DashboardProps {
  username: string | null;
  role: UserRole | null;
  teams: UserTeam[];
  onLogout: () => void;
  onChangePassword: () => void;
}

export function Dashboard({
  username,
  role,
  teams,
  onLogout,
  onChangePassword,
}: DashboardProps) {
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const { data, isLoading, error } = useSites(
    page,
    perPage,
    debouncedSearch || undefined,
  );
  const createSite = useCreateSite();
  const deleteSite = useDeleteSite();
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [siteToDelete, setSiteToDelete] = useState<SiteResponse | null>(null);
  const [selectedTeamId, setSelectedTeamId] = useState<number | null>(null);

  const totalPages = data ? Math.ceil(data.total / data.per_page) : 0;
  const hasNoSites = data != null && data.total === 0 && !debouncedSearch;

  const sites = data?.data ?? [];
  const filteredSites = selectedTeamId
    ? sites.filter((s) => s.team_id === selectedTeamId)
    : sites;
  const orphanedCount = sites.filter((s) => s.team_id == null).length;

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  return (
    <div style={pageWrapper}>
      <div style={headerRow}>
        <h1 style={pageTitle}>Site Oxidation</h1>
        <UserMenu
          username={username ?? ""}
          isAdmin={role === "admin"}
          onChangePassword={onChangePassword}
          onLogout={onLogout}
        />
      </div>

      <SearchToolbar
        action={
          <FormToggleButton
            isOpen={showCreateForm}
            openLabel="Add Site"
            onClick={() => {
              createSite.reset();
              setShowCreateForm(!showCreateForm);
            }}
          />
        }
      >
        {!hasNoSites && (
          <SearchInput
            value={searchInput}
            onChange={setSearchInput}
            placeholder="Search sites..."
          />
        )}
      </SearchToolbar>

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

      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : data ? (
        <>
          {role === "admin" && orphanedCount > 0 && (
            <p style={mutedText}>
              {orphanedCount} site{orphanedCount === 1 ? "" : "s"} with no team
              assigned
            </p>
          )}
          {role !== "admin" && teams.length > 1 && (
            <select
              value={selectedTeamId ?? ""}
              onChange={(e) =>
                setSelectedTeamId(
                  e.target.value ? Number(e.target.value) : null,
                )
              }
              style={compactInput}
            >
              <option value="">All teams</option>
              {teams.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </select>
          )}
          <SiteList
            sites={filteredSites}
            onDelete={(site) => setSiteToDelete(site)}
          />
          <Pagination
            page={data.page}
            totalPages={totalPages}
            onPageChange={goToPage}
          />
        </>
      ) : null}

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
