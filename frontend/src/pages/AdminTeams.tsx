import { useEffect, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import { AdminNav } from "../components/ui/AdminNav";
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
import { FormInput } from "../components/ui/FormControls";
import { FormToggleButton } from "../components/ui/FormToggleButton";
import {
  pageWrapper,
  backLink,
  table,
  tableHeaderRow,
  tableRow,
  tableCellLeft,
  tableCellCenter,
  tableCell,
  inlineForm,
  compactInput,
  mutedText,
} from "../lib/styles";
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

  useEffect(() => {
    if (teams && teams.data.length === 0 && teams.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [teams, page, goToPage]);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  return (
    <div style={pageWrapper}>
      <Link to="/" style={backLink}>
        &larr; Back to Dashboard
      </Link>
      <AdminNav />

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
        <SearchInput
          value={searchInput}
          onChange={setSearchInput}
          placeholder="Search teams..."
        />
      </SearchToolbar>

      {showCreateForm && (
        <div>
          <form onSubmit={handleCreateTeam} style={inlineForm}>
            <FormInput
              type="text"
              placeholder="New team name"
              value={newTeamName}
              onChange={(e) => setNewTeamName(e.target.value)}
              required
              style={{ flex: 1 }}
            />
            <button
              type="submit"
              disabled={createTeam.isPending}
              style={compactInput}
            >
              {createTeam.isPending ? "Creating..." : "Create Team"}
            </button>
            <button
              type="button"
              onClick={() => {
                createTeam.reset();
                setNewTeamName("");
                setShowCreateForm(false);
              }}
              style={compactInput}
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
          <table style={table}>
            <thead>
              <tr style={tableHeaderRow}>
                <th style={tableCellLeft}>Name</th>
                <th style={tableCellCenter}>Members</th>
                <th style={tableCellCenter}>Sites</th>
                <th style={tableCellLeft}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {teamList.map((team) => (
                <tr key={team.id} style={tableRow}>
                  <td style={tableCellLeft}>
                    {editingTeam?.id === team.id ? (
                      <form
                        onSubmit={handleRename}
                        style={{ display: "flex", gap: "8px" }}
                      >
                        <input
                          type="text"
                          value={editName}
                          onChange={(e) => setEditName(e.target.value)}
                          required
                          style={compactInput}
                        />
                        <button type="submit" style={compactInput}>
                          Save
                        </button>
                        <button
                          type="button"
                          onClick={() => setEditingTeam(null)}
                          style={compactInput}
                        >
                          Cancel
                        </button>
                      </form>
                    ) : (
                      <Link to={`/admin/teams/${team.id}`}>{team.name}</Link>
                    )}
                  </td>
                  <td style={tableCellCenter}>{team.member_count}</td>
                  <td style={tableCellCenter}>{team.site_count}</td>
                  <td style={tableCell}>
                    <div style={{ display: "flex", gap: "8px" }}>
                      <button
                        onClick={() =>
                          setManagingTeam(
                            managingTeam?.id === team.id ? null : team,
                          )
                        }
                        style={compactInput}
                      >
                        {managingTeam?.id === team.id ? "Close" : "Members"}
                      </button>
                      <button
                        onClick={() => startEditing(team)}
                        style={compactInput}
                      >
                        Rename
                      </button>
                      <button
                        onClick={() => setTeamToDelete(team)}
                        style={{
                          ...compactInput,
                          color: "#dc2626",
                          background: "none",
                          border: "none",
                          cursor: "pointer",
                        }}
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
        <p style={mutedText}>No teams yet. Create one above.</p>
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
        border: "1px solid #333",
        borderRadius: "8px",
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
                  <span style={mutedText}> ({user.role})</span>
                </span>
                <button
                  onClick={() =>
                    removeMember.mutate({ teamId, userId: user.id })
                  }
                  style={{
                    color: "#dc2626",
                    background: "none",
                    border: "none",
                    cursor: "pointer",
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
        <p style={{ ...mutedText, margin: "0 0 12px 0", fontSize: "14px" }}>
          No members yet
        </p>
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
        <input
          type="text"
          placeholder="Search users to add..."
          value={searchInput}
          onChange={(e) => {
            setSearchInput(e.target.value);
            setSelectedUserId("");
          }}
          style={compactInput}
        />
        {debouncedSearch.trim() && (
          <div style={{ display: "flex", gap: "8px" }}>
            <select
              value={selectedUserId}
              onChange={(e) => setSelectedUserId(e.target.value)}
              style={{ ...compactInput, flex: 1 }}
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
            </select>
            <button
              onClick={handleAddMember}
              disabled={!selectedUserId || addMember.isPending}
              style={compactInput}
            >
              Add
            </button>
          </div>
        )}
        {hasMoreResults && debouncedSearch.trim() && (
          <p style={{ ...mutedText, fontSize: "12px", margin: 0 }}>
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
