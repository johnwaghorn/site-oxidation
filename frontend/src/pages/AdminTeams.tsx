import { useEffect, useState, type FormEvent } from "react";
import { Link, useNavigate } from "react-router-dom";
import {
  useAdminTeams,
  useCreateTeam,
  useUpdateTeam,
  useDeleteTeam,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { Modal } from "../components/ui/Modal";
import { Pagination } from "../components/ui/Pagination";
import { SearchInput } from "../components/ui/SearchInput";
import { TableCard, TableCardEmpty } from "../components/ui/TableCard";
import { FormInput } from "../components/ui/FormControls";
import { Truncate } from "../components/ui/Truncate";
import type { components } from "../generated/schema";

type TeamResponse = components["schemas"]["TeamResponse"];

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

  const closeCreateForm = () => {
    createTeam.reset();
    setNewTeamName("");
    setShowCreateForm(false);
  };

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

  const closeRename = () => {
    updateTeam.reset();
    setEditingTeam(null);
  };

  const teamList = teams?.data ?? [];
  const totalPages = teams ? Math.ceil(teams.total / teams.per_page) : 0;
  const hasNoTeams =
    teams != null && !error && teams.total === 0 && !debouncedSearch;

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
      <div className="page-header">
        <h1 className="page-title">Teams</h1>
        <button
          type="button"
          className="button-primary-action"
          aria-haspopup="dialog"
          onClick={() => {
            createTeam.reset();
            setShowCreateForm(true);
          }}
        >
          Add team
        </button>
      </div>

      {deleteTeam.isError && <ErrorMessage error={deleteTeam.error} />}

      <Modal isOpen={showCreateForm} onClose={closeCreateForm} title="Add team">
        <form onSubmit={handleCreateTeam} className="form-column">
          <label className="field-label">
            <span className="field-label-text">Team name</span>
            <FormInput
              type="text"
              placeholder="New team name"
              value={newTeamName}
              onChange={(e) => setNewTeamName(e.target.value)}
              required
              maxLength={60}
            />
          </label>
          {createTeam.isError && <ErrorMessage error={createTeam.error} />}
          <div
            style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}
          >
            <button
              type="button"
              className="button-secondary-action"
              onClick={closeCreateForm}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="button-primary-action"
              disabled={createTeam.isPending}
            >
              {createTeam.isPending ? "Creating..." : "Create Team"}
            </button>
          </div>
        </form>
      </Modal>

      {hasNoTeams ? (
        <p className="muted-text">
          No teams yet. Create one above. Maybe a team just for admins!
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
            {isLoading ? (
              <LoadingSpinner />
            ) : error ? (
              <ErrorMessage error={error} />
            ) : teamList.length > 0 ? (
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
                        <Link to={`/admin/teams/${team.id}`}>
                          <Truncate text={team.name} />
                        </Link>
                      </td>
                      <td className="table-cell-center">{team.member_count}</td>
                      <td className="table-cell-center">{team.site_count}</td>
                      <td className="table-cell">
                        <div style={{ display: "flex", gap: "8px" }}>
                          <button
                            className="button-table-action compact-input"
                            aria-haspopup="dialog"
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
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <TableCardEmpty>No teams match your search.</TableCardEmpty>
            )}
          </TableCard>
          {teamList.length > 0 && (
            <Pagination
              page={page}
              totalPages={totalPages}
              onPageChange={goToPage}
            />
          )}
        </>
      )}

      <Modal
        isOpen={editingTeam !== null}
        onClose={closeRename}
        title="Rename team"
      >
        <form onSubmit={handleRename} className="form-column">
          <label className="field-label">
            <span className="field-label-text">Team name</span>
            <FormInput
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              required
              maxLength={60}
            />
          </label>
          {updateTeam.isError && <ErrorMessage error={updateTeam.error} />}
          <div
            style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}
          >
            <button
              type="button"
              className="button-secondary-action"
              onClick={closeRename}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="button-primary-action"
              disabled={updateTeam.isPending}
            >
              {updateTeam.isPending ? "Saving..." : "Save"}
            </button>
          </div>
        </form>
      </Modal>

      <ConfirmDialog
        isOpen={teamToDelete !== null}
        onClose={() => setTeamToDelete(null)}
        onConfirm={() => {
          if (!teamToDelete) return;
          if (teamToDelete.site_count > 0) {
            navigate(`/admin/teams/${teamToDelete.id}/sites`);
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
