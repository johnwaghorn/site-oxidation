import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import {
  useAddTeamMember,
  useAdminTeams,
  useAdminUser,
  useRemoveTeamMember,
} from "../hooks/useAdmin";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { usePagination } from "../hooks/usePagination";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { FormInput } from "../components/ui/FormControls";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { Modal } from "../components/ui/Modal";
import { Pagination } from "../components/ui/Pagination";
import { SearchInput } from "../components/ui/SearchInput";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import { Truncate } from "../components/ui/Truncate";
import type { components } from "../generated/schema";

type TeamResponse = components["schemas"]["TeamResponse"];

export function AdminUserDetail() {
  const { id } = useParams<{ id: string }>();
  const userId = Number(id);
  const {
    data: user,
    isLoading: userLoading,
    error: userError,
  } = useAdminUser(userId);
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const {
    data: teams,
    isLoading: teamsLoading,
    error: teamsError,
  } = useAdminTeams(page, perPage, debouncedSearch || undefined, {
    memberId: userId,
  });
  const removeMember = useRemoveTeamMember();
  const [isAdding, setIsAdding] = useState(false);
  const [teamToLeave, setTeamToLeave] = useState<TeamResponse | null>(null);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  useEffect(() => {
    if (teams && teams.data.length === 0 && teams.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [teams, page, goToPage]);

  if (!Number.isInteger(userId) || userId <= 0) {
    return <ErrorMessage error={new Error("User not found")} />;
  }
  if (userLoading) return <LoadingSpinner />;
  if (userError) return <ErrorMessage error={userError} />;
  if (!user) return <ErrorMessage error={new Error("User not found")} />;

  const totalPages = teams ? Math.ceil(teams.total / teams.per_page) : 0;
  const hasNoTeams =
    teams != null && !teamsError && teams.total === 0 && !debouncedSearch;

  return (
    <div className="page-wrapper">
      <Link to="/admin/users" className="back-link">
        &larr; Back to Users
      </Link>

      <h1 className="page-title" style={{ marginBottom: "8px" }}>
        <Truncate text={user.username} maxWidth="90%" />
      </h1>
      <dl className="detail-list">
        <div>
          <dt>Role</dt>
          <dd>{user.role === "admin" ? "Admin" : "User"}</dd>
        </div>
        <div>
          <dt>Active</dt>
          <dd>{user.active ? "Yes" : "No"}</dd>
        </div>
        <div>
          <dt>Teams</dt>
          <dd>{user.team_count}</dd>
        </div>
      </dl>

      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "12px",
          margin: "24px 0 16px",
        }}
      >
        <h2 style={{ margin: 0 }}>Teams</h2>
        <button
          type="button"
          className="button-primary-action"
          aria-haspopup="dialog"
          onClick={() => setIsAdding(true)}
        >
          Add to team
        </button>
      </div>

      {removeMember.isError && <ErrorMessage error={removeMember.error} />}
      {hasNoTeams ? (
        <p className="muted-text">
          {user.username} is not in any teams yet.
          {user.role === "user" &&
            " Users only see sites that belong to their teams."}
        </p>
      ) : (
        <>
          <TableCard
            toolbar={
              <SearchInput
                value={searchInput}
                onChange={setSearchInput}
                placeholder="Search teams..."
              />
            }
          >
            {teamsLoading ? (
              <LoadingSpinner />
            ) : teamsError ? (
              <ErrorMessage error={teamsError} />
            ) : teams && teams.data.length > 0 ? (
              <table className="data-table">
                <thead>
                  <tr className="table-header-row">
                    <th className="table-cell-left">Team</th>
                    <th className="table-cell-center">Members</th>
                    <th className="table-cell-center">Sites</th>
                    <th className="table-cell-right">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {teams.data.map((team) => (
                    <tr key={team.id} className="table-row">
                      <td className="table-cell">
                        <Link to={`/admin/teams/${team.id}`}>
                          <Truncate text={team.name} />
                        </Link>
                      </td>
                      <td className="table-cell-center">{team.member_count}</td>
                      <td className="table-cell-center">{team.site_count}</td>
                      <td className="table-cell-right">
                        <button
                          className="button-table-action button-table-danger compact-input"
                          onClick={() => setTeamToLeave(team)}
                        >
                          Remove
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <TableCardEmpty>No teams match your search.</TableCardEmpty>
            )}
          </TableCard>
          {teams && teams.data.length > 0 && (
            <Pagination
              page={page}
              totalPages={totalPages}
              onPageChange={goToPage}
            />
          )}
        </>
      )}

      <Modal
        isOpen={isAdding}
        onClose={() => setIsAdding(false)}
        title={`Add ${user.username} to a team`}
      >
        <AddToTeamForm userId={user.id} onDone={() => setIsAdding(false)} />
      </Modal>

      <ConfirmDialog
        isOpen={teamToLeave !== null}
        onClose={() => setTeamToLeave(null)}
        onConfirm={() =>
          teamToLeave &&
          removeMember.mutate({ teamId: teamToLeave.id, userId: user.id })
        }
        title="Remove from Team"
        message={`Remove "${user.username}" from "${teamToLeave?.name}"? They will lose access to this team's sites straight away.`}
        confirmText="Remove"
        isDestructive
      />
    </div>
  );
}

function AddToTeamForm({
  userId,
  onDone,
}: {
  userId: number;
  onDone: () => void;
}) {
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const { data: candidates } = useAdminTeams(
    1,
    20,
    debouncedSearch || undefined,
    { excludeMemberId: userId },
  );
  const addMember = useAddTeamMember();

  const teams = candidates?.data ?? [];
  const total = candidates?.total ?? 0;

  return (
    <div className="form-column">
      <label className="field-label">
        <span className="field-label-text">Find a team</span>
        <FormInput
          type="search"
          placeholder="Search teams to join..."
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
        />
      </label>
      {teams.length > 0 ? (
        <ul className="member-candidates">
          {teams.map((team) => (
            <li key={team.id}>
              <Truncate text={team.name} />
              <button
                type="button"
                className="button-table-action compact-input"
                aria-label={`Add to ${team.name}`}
                onClick={() =>
                  addMember.mutate({
                    teamId: team.id,
                    member: { user_id: userId },
                  })
                }
                disabled={addMember.isPending}
              >
                Add
              </button>
            </li>
          ))}
        </ul>
      ) : (
        <p className="muted-text" style={{ margin: 0 }}>
          {debouncedSearch
            ? "No matching teams."
            : "They are already in every team."}
        </p>
      )}
      {total > teams.length && (
        <p className="muted-text" style={{ margin: 0, fontSize: "12px" }}>
          Showing {teams.length} of {total} teams. Refine your search.
        </p>
      )}
      {addMember.isError && <ErrorMessage error={addMember.error} />}
      <div style={{ display: "flex", justifyContent: "flex-end" }}>
        <button
          type="button"
          className="button-secondary-action"
          onClick={onDone}
        >
          Done
        </button>
      </div>
    </div>
  );
}
