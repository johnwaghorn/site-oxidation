import { useEffect, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  useAdminTeams,
  useAdminUsers,
  useCreateTeam,
  useUpdateTeam,
  useDeleteTeam,
  useAddTeamMember,
  useRemoveTeamMember,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { Pagination } from "../components/ui/Pagination";
import { SearchInput, SearchToolbar } from "../components/ui/SearchInput";
import { FormInput, FormSelect } from "../components/ui/FormControls";
import { FormToggleButton } from "../components/ui/FormToggleButton";
import { Truncate } from "../components/ui/Truncate";
import type { components } from "../generated/schema";

type TeamResponse = components["schemas"]["TeamResponse"];
type UserResponse = components["schemas"]["UserResponse"];

export function AdminTeams() {
  const navigate = useNavigate();
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const {
    data: teams,
    isLoading,
    error,
  } = useAdminTeams(page, perPage, debouncedSearch || undefined);
  const createTeam = useCreateTeam();
  const updateTeam = useUpdateTeam();
  const deleteTeam = useDeleteTeam();

  const [newTeamName, setNewTeamName] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [editingTeam, setEditingTeam] = useState<TeamResponse | null>(null);
  const [editName, setEditName] = useState("");
  const [teamToDelete, setTeamToDelete] = useState<TeamResponse | null>(null);
  const [managingTeam, setManagingTeam] = useState<TeamResponse | null>(null);

  const handleCreateTeam = (e: FormEvent) => {
    e.preventDefault();
    if (!newTeamName.trim()) return;
    createTeam.mutate(
      { name: newTeamName.trim() },
      {
        onSuccess: () => {
          setNewTeamName("");
          setShowCreateForm(false);
        },
      },
    );
  };

  const handleRename = (e: FormEvent) => {
    e.preventDefault();
    if (!editingTeam || !editName.trim()) return;
    updateTeam.mutate(
      { id: editingTeam.id, team: { name: editName.trim() } },
      { onSuccess: () => setEditingTeam(null) },
    );
  };

  const startEditing = (team: TeamResponse) => {
    updateTeam.reset();
    setEditingTeam(team);
    setEditName(team.name);
  };

  const teamList = teams?.data ?? [];
  const totalPages = teams ? Math.ceil(teams.total / teams.per_page) : 0;
  const hasNoTeams = teams != null && teams.total === 0 && !debouncedSearch;

  useEffect(() => {
    if (teams && teams.data.length === 0 && teams.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [teams, page, goToPage]);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Teams</h1>

      {editingTeam && updateTeam.isError && (
        <ErrorMessage error={updateTeam.error} />
      )}
      {deleteTeam.isError && <ErrorMessage error={deleteTeam.error} />}

      <SearchToolbar
        action={
          <FormToggleButton
            isOpen={showCreateForm}
            openLabel="Create Team"
            onClick={() => {
              createTeam.reset();
              if (showCreateForm) {
                setNewTeamName("");
              }
              setShowCreateForm(!showCreateForm);
            }}
          />
        }
      >
        {!hasNoTeams && (
          <SearchInput
            value={searchInput}
            onChange={setSearchInput}
            placeholder="Search teams..."
          />
        )}
      </SearchToolbar>

      {showCreateForm && (
        <div>
          <form onSubmit={handleCreateTeam} className="inline-form">
            <FormInput
              type="text"
              placeholder="New team name"
              value={newTeamName}
              onChange={(e) => setNewTeamName(e.target.value)}
              required
              maxLength={60}
              style={{ flex: 1 }}
            />
            <button
              type="submit"
              className="button-primary-action compact-input"
              disabled={createTeam.isPending}
            >
              {createTeam.isPending ? "Creating..." : "Create Team"}
            </button>
            <button
              type="button"
              className="button-secondary-action compact-input"
              onClick={() => {
                createTeam.reset();
                setNewTeamName("");
                setShowCreateForm(false);
              }}
            >
              Cancel
            </button>
          </form>
          {createTeam.isError && <ErrorMessage error={createTeam.error} />}
        </div>
      )}

      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : teamList.length > 0 ? (
        <>
          <table className="data-table">
            <thead>
              <tr className="table-header-row">
                <th className="table-cell-left">Name</th>
                <th className="table-cell-center">Members</th>
                <th className="table-cell-center">Sites</th>
                <th className="table-cell-left">Actions</th>
              </tr>
            </thead>
            <tbody>
              {teamList.map((team) => (
                <tr key={team.id} className="table-row">
                  <td className="table-cell-left">
                    {editingTeam?.id === team.id ? (
                      <form
                        onSubmit={handleRename}
                        style={{ display: "flex", gap: "8px" }}
                      >
                        <FormInput
                          type="text"
                          value={editName}
                          onChange={(e) => setEditName(e.target.value)}
                          required
                          maxLength={60}
                          className="compact-input"
                          style={{ minWidth: "180px" }}
                        />
                        <button
                          type="submit"
                          className="button-table-action compact-input"
                        >
                          Save
                        </button>
                        <button
                          type="button"
                          className="button-table-action compact-input"
                          onClick={() => setEditingTeam(null)}
                        >
                          Cancel
                        </button>
                      </form>
                    ) : (
                      <Link to={`/admin/teams/${team.id}`}>
                        <Truncate text={team.name} />
                      </Link>
                    )}
                  </td>
                  <td className="table-cell-center">{team.member_count}</td>
                  <td className="table-cell-center">{team.site_count}</td>
                  <td className="table-cell">
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button
                        className="button-table-action compact-input"
                        onClick={() =>
                          setManagingTeam(
                            managingTeam?.id === team.id ? null : team,
                          )
                        }
                      >
                        {managingTeam?.id === team.id ? "Close" : "Members"}
                      </button>
                      <button
                        className="button-table-action compact-input"
                        onClick={() => startEditing(team)}
                      >
                        Rename
                      </button>
                      <button
                        className="button-table-action button-table-danger compact-input"
                        onClick={() => setTeamToDelete(team)}
                      >
                        Delete
                      </button>
                    </div>
                    {managingTeam?.id === team.id && (
                      <MemberPanel teamId={team.id} />
                    )}
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
        <p className="muted-text">
          No teams yet. Create one above. Maybe a team just for admins!
        </p>
      )}

      <ConfirmDialog
        isOpen={teamToDelete !== null}
        onClose={() => setTeamToDelete(null)}
        onConfirm={() => {
          if (!teamToDelete) return;
          if (teamToDelete.site_count > 0) {
            navigate(`/admin/teams/${teamToDelete.id}`);
            return;
          }
          deleteTeam.mutate(teamToDelete.id);
        }}
        title="Delete Team"
        message={
          teamToDelete?.site_count
            ? `"${teamToDelete.name}" still has ${teamToDelete.site_count} assigned site(s). Open the team details to reassign them or remove them from the team before deleting it.`
            : `Are you sure you want to delete "${teamToDelete?.name}"?`
        }
        confirmText={teamToDelete?.site_count ? "Manage Sites" : "Delete"}
        cancelText={teamToDelete?.site_count ? "Close" : "Cancel"}
        isDestructive={!teamToDelete?.site_count}
      />
    </div>
  );
}

interface MemberPanelProps {
  teamId: number;
}

function MemberPanel({ teamId }: MemberPanelProps) {
  const [searchInput, setSearchInput] = useState("");
  const [selectedUserId, setSelectedUserId] = useState<string>("");
  const debouncedSearch = useDebouncedValue(searchInput);
  const {
    page: membersPage,
    perPage: membersPerPage,
    goToPage: goToMembersPage,
  } = usePagination();

  const { data: membersData } = useAdminUsers({
    page: membersPage,
    perPage: membersPerPage,
    teamId,
    active: true,
  });
  const { data: candidatesData } = useAdminUsers({
    page: 1,
    perPage: 20,
    search: debouncedSearch.trim() || undefined,
    excludeTeamId: teamId,
    active: true,
  });

  const addMember = useAddTeamMember();
  const removeMember = useRemoveTeamMember();

  const members = membersData?.data ?? [];
  const memberTotalPages = membersData
    ? Math.ceil(membersData.total / membersData.per_page)
    : 0;

  useEffect(() => {
    if (
      membersData &&
      membersData.data.length === 0 &&
      membersData.total > 0 &&
      membersPage > 1
    ) {
      goToMembersPage(1);
    }
  }, [membersData, membersPage, goToMembersPage]);

  const candidates = candidatesData?.data ?? [];
  const totalCandidates = candidatesData?.total ?? 0;
  const hasMoreResults = totalCandidates > candidates.length;

  const handleAddMember = () => {
    if (!selectedUserId) return;
    addMember.mutate(
      { teamId, member: { user_id: Number(selectedUserId) } },
      {
        onSuccess: () => {
          setSelectedUserId("");
          setSearchInput("");
        },
      },
    );
  };

  return (
    <div
      style={{
        marginTop: "12px",
        padding: "12px",
        border: "1px solid var(--color-border)",
        borderRadius: "12px",
        backgroundColor: "var(--color-surface-muted)",
      }}
    >
      <div style={{ marginBottom: "8px", fontWeight: 500 }}>Members</div>
      {members.length > 0 ? (
        <>
          <ul style={{ listStyle: "none", padding: 0, margin: "0 0 12px 0" }}>
            {members.map((user: UserResponse) => (
              <li
                key={user.id}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  padding: "4px 0",
                }}
              >
                <span>
                  {user.username}
                  <span className="muted-text"> ({user.role})</span>
                </span>
                <button
                  className="button-table-action button-table-danger"
                  onClick={() =>
                    removeMember.mutate({ teamId, userId: user.id })
                  }
                  style={{
                    padding: "4px 8px",
                    fontSize: "12px",
                  }}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
          <Pagination
            page={membersPage}
            totalPages={memberTotalPages}
            onPageChange={goToMembersPage}
          />
        </>
      ) : (
        <p
          className="muted-text"
          style={{ margin: "0 0 12px 0", fontSize: "14px" }}
        >
          No members yet
        </p>
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
        <FormInput
          type="text"
          placeholder="Search users to add..."
          value={searchInput}
          onChange={(e) => {
            setSearchInput(e.target.value);
            setSelectedUserId("");
          }}
          className="compact-input"
          style={{ width: "100%" }}
        />
        {debouncedSearch.trim() && (
          <div style={{ display: "flex", gap: "8px" }}>
            <FormSelect
              value={selectedUserId}
              onChange={(e) => setSelectedUserId(e.target.value)}
              className="compact-input"
              style={{ flex: 1 }}
            >
              <option value="">
                {candidates.length > 0
                  ? `Select user (${candidates.length} match${candidates.length === 1 ? "" : "es"})`
                  : "No matches"}
              </option>
              {candidates.map((user) => (
                <option key={user.id} value={user.id}>
                  {user.username}
                </option>
              ))}
            </FormSelect>
            <button
              className="button-table-action compact-input"
              onClick={handleAddMember}
              disabled={!selectedUserId || addMember.isPending}
            >
              Add
            </button>
          </div>
        )}
        {hasMoreResults && debouncedSearch.trim() && (
          <p className="muted-text" style={{ fontSize: "12px", margin: 0 }}>
            Showing {candidatesData?.data.length} of {totalCandidates} results.
            Please refine your search
          </p>
        )}
      </div>
      {addMember.isError && <ErrorMessage error={addMember.error} />}
      {removeMember.isError && <ErrorMessage error={removeMember.error} />}
    </div>
  );
}
