import { useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
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
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
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
  const { data: teams, isLoading, error } = useAdminTeams();
  const { data: users } = useAdminUsers();
  const createTeam = useCreateTeam();
  const updateTeam = useUpdateTeam();
  const deleteTeam = useDeleteTeam();
  const addMember = useAddTeamMember();
  const removeMember = useRemoveTeamMember();

  const [newTeamName, setNewTeamName] = useState("");
  const [editingTeam, setEditingTeam] = useState<TeamResponse | null>(null);
  const [editName, setEditName] = useState("");
  const [teamToDelete, setTeamToDelete] = useState<TeamResponse | null>(null);
  const [managingTeam, setManagingTeam] = useState<TeamResponse | null>(null);
  const [selectedUserId, setSelectedUserId] = useState<string>("");

  const handleCreateTeam = (e: FormEvent) => {
    e.preventDefault();
    if (!newTeamName.trim()) return;
    createTeam.mutate(
      { name: newTeamName.trim() },
      {
        onSuccess: () => setNewTeamName(""),
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
    setEditingTeam(team);
    setEditName(team.name);
  };

  const teamMembers = (teamId: number): UserResponse[] => {
    if (!users) return [];
    return users.filter((u) =>
      u.team_names.split(", ").some((t) => {
        const team = teams?.find((tm) => tm.id === teamId);
        return team && t === team.name;
      }),
    );
  };

  const nonMembers = (teamId: number): UserResponse[] => {
    if (!users) return [];
    const members = teamMembers(teamId);
    const memberIds = new Set(members.map((m) => m.id));
    return users.filter((u) => !memberIds.has(u.id));
  };

  const handleAddMember = () => {
    if (!managingTeam || !selectedUserId) return;
    addMember.mutate(
      { teamId: managingTeam.id, member: { user_id: Number(selectedUserId) } },
      { onSuccess: () => setSelectedUserId("") },
    );
  };

  return (
    <div style={pageWrapper}>
      <Link to="/" style={backLink}>
        &larr; Back to Dashboard
      </Link>
      <AdminNav />

      <form onSubmit={handleCreateTeam} style={inlineForm}>
        <input
          type="text"
          placeholder="New team name"
          value={newTeamName}
          onChange={(e) => setNewTeamName(e.target.value)}
          required
          style={{ ...compactInput, flex: 1 }}
        />
        <button
          type="submit"
          disabled={createTeam.isPending}
          style={compactInput}
        >
          {createTeam.isPending ? "Creating..." : "Create Team"}
        </button>
      </form>
      {createTeam.isError && <ErrorMessage error={createTeam.error} />}

      {isLoading ? (
        <LoadingSpinner />
      ) : error ? (
        <ErrorMessage error={error} />
      ) : teams && teams.length > 0 ? (
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
            {teams.map((team) => (
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
                    team.name
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
                    <MemberPanel
                      members={teamMembers(team.id)}
                      nonMembers={nonMembers(team.id)}
                      selectedUserId={selectedUserId}
                      onSelectUser={setSelectedUserId}
                      onAddMember={handleAddMember}
                      onRemoveMember={(userId) =>
                        removeMember.mutate({
                          teamId: team.id,
                          userId,
                        })
                      }
                      isAdding={addMember.isPending}
                    />
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <p style={mutedText}>No teams yet. Create one above.</p>
      )}

      <ConfirmDialog
        isOpen={teamToDelete !== null}
        onClose={() => setTeamToDelete(null)}
        onConfirm={() => teamToDelete && deleteTeam.mutate(teamToDelete.id)}
        title="Delete Team"
        message={
          teamToDelete?.site_count
            ? `Cannot delete "${teamToDelete.name}" because it has ${teamToDelete.site_count} assigned site(s). Reassign them first.`
            : `Are you sure you want to delete "${teamToDelete?.name}"?`
        }
        confirmText="Delete"
        isDestructive
      />
    </div>
  );
}

interface MemberPanelProps {
  members: UserResponse[];
  nonMembers: UserResponse[];
  selectedUserId: string;
  onSelectUser: (id: string) => void;
  onAddMember: () => void;
  onRemoveMember: (userId: number) => void;
  isAdding: boolean;
}

function MemberPanel({
  members,
  nonMembers,
  selectedUserId,
  onSelectUser,
  onAddMember,
  onRemoveMember,
  isAdding,
}: MemberPanelProps) {
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
        <ul style={{ listStyle: "none", padding: 0, margin: "0 0 12px 0" }}>
          {members.map((user) => (
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
                onClick={() => onRemoveMember(user.id)}
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
      ) : (
        <p style={{ ...mutedText, margin: "0 0 12px 0", fontSize: "14px" }}>
          No members yet
        </p>
      )}
      {nonMembers.length > 0 && (
        <div style={{ display: "flex", gap: "8px" }}>
          <select
            value={selectedUserId}
            onChange={(e) => onSelectUser(e.target.value)}
            style={{ ...compactInput, flex: 1 }}
          >
            <option value="">Select user...</option>
            {nonMembers.map((user) => (
              <option key={user.id} value={user.id}>
                {user.username}
              </option>
            ))}
          </select>
          <button
            onClick={onAddMember}
            disabled={!selectedUserId || isAdding}
            style={compactInput}
          >
            Add
          </button>
        </div>
      )}
    </div>
  );
}
