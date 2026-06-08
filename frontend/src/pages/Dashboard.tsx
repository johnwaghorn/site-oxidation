import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
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
import { FormSelect } from "../components/ui/FormControls";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import type { ThemePreference } from "../hooks/useThemePreference";
import { pageWrapper, headerRow, pageTitle, mutedText } from "../lib/styles";
import type { components } from "../generated/schema";

type SiteResponse = components["schemas"]["SiteResponse"];
type UserTeam = components["schemas"]["UserTeam"];
type UserRole = components["schemas"]["UserRole"];

interface DashboardProps {
  username: string | null;
  role: UserRole | null;
  teams: UserTeam[];
  themePreference: ThemePreference;
  onLogout: () => void;
  onChangePassword: () => void;
  onThemePreferenceChange: (preference: ThemePreference) => void;
}

export function Dashboard({
  username,
  role,
  teams,
  themePreference,
  onLogout,
  onChangePassword,
  onThemePreferenceChange,
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
  const showGetStarted = role === "admin" && hasNoSites;

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
          themePreference={themePreference}
          onChangePassword={onChangePassword}
          onLogout={onLogout}
          onThemePreferenceChange={onThemePreferenceChange}
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
        showGetStarted ? (
          !showCreateForm && (
            <GetStartedNudge onAddSite={() => setShowCreateForm(true)} />
          )
        ) : (
          <>
            {role === "admin" && orphanedCount > 0 && (
              <p style={mutedText}>
                {orphanedCount} site{orphanedCount === 1 ? "" : "s"} with no
                team assigned
              </p>
            )}
            {role !== "admin" && teams.length > 1 && (
              <FormSelect
                value={selectedTeamId ?? ""}
                onChange={(e) =>
                  setSelectedTeamId(
                    e.target.value ? Number(e.target.value) : null,
                  )
                }
                style={{ marginBottom: "16px" }}
              >
                <option value="">All teams</option>
                {teams.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </FormSelect>
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
        )
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

function GetStartedNudge({ onAddSite }: { onAddSite: () => void }) {
  return (
    <div
      style={{
        boxSizing: "border-box",
        maxWidth: "720px",
        marginTop: "8px",
        padding: "28px",
        border: "1px solid var(--color-border)",
        borderRadius: "14px",
        background:
          "linear-gradient(135deg, var(--color-primary-soft), transparent 48%), var(--color-surface)",
        boxShadow: "var(--shadow-card)",
      }}
    >
      <p
        style={{
          ...mutedText,
          margin: "0 0 8px 0",
          fontSize: "13px",
          fontWeight: 600,
          letterSpacing: "0.04em",
          textTransform: "uppercase",
        }}
      >
        Welcome
      </p>
      <h2 style={{ margin: "0 0 10px 0", fontSize: "28px", lineHeight: 1.2 }}>
        Start monitoring your first site
      </h2>
      <p style={{ ...mutedText, maxWidth: "560px", margin: "0 0 22px 0" }}>
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
