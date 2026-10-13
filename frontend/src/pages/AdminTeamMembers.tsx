import { useEffect, useState } from "react";
import { useOutletContext } from "react-router-dom";
import {
  useAddTeamMember,
  useAdminUsers,
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
import type { components } from "../generated/schema";

type TeamResponse = components["schemas"]["TeamResponse"];
type UserResponse = components["schemas"]["UserResponse"];

export function AdminTeamMembers() {
  const team = useOutletContext<TeamResponse>();
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const {
    data: members,
    isLoading,
    error,
  } = useAdminUsers({
    page,
    perPage,
    search: debouncedSearch || undefined,
    teamId: team.id,
    active: true,
  });
  const removeMember = useRemoveTeamMember();
  const [isAdding, setIsAdding] = useState(false);
  const [memberToRemove, setMemberToRemove] = useState<UserResponse | null>(
    null,
  );

  useEffect(() => {
    if (members && members.data.length === 0 && members.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [members, page, goToPage]);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  const totalPages = members ? Math.ceil(members.total / members.per_page) : 0;
  const hasNoMembers =
    members != null && !error && members.total === 0 && !debouncedSearch;

  return (
    <>
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          justifyContent: "space-between",
          gap: "12px",
          marginBottom: "16px",
        }}
      >
        <p className="muted-text" style={{ margin: 0 }}>
          Members can see and manage this team&apos;s sites.
        </p>
        <button
          type="button"
          className="button-primary-action"
          aria-haspopup="dialog"
          onClick={() => setIsAdding(true)}
        >
          Add member
        </button>
      </div>

      {removeMember.isError && <ErrorMessage error={removeMember.error} />}
      {hasNoMembers ? (
        <p className="muted-text">This team has no members yet.</p>
      ) : (
        <>
          <TableCard
            toolbar={
              <SearchInput
                value={searchInput}
                onChange={setSearchInput}
                placeholder="Search members..."
              />
            }
          >
            {isLoading ? (
              <LoadingSpinner />
            ) : error ? (
              <ErrorMessage error={error} />
            ) : members && members.data.length > 0 ? (
              <table className="data-table">
                <thead>
                  <tr className="table-header-row">
                    <th className="table-cell-left">Username</th>
                    <th className="table-cell-left">Role</th>
                    <th className="table-cell-right">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {members.data.map((user) => (
                    <tr key={user.id} className="table-row">
                      <td className="table-cell">{user.username}</td>
                      <td className="table-cell">{user.role}</td>
                      <td className="table-cell-right">
                        <button
                          className="button-table-action button-table-danger compact-input"
                          onClick={() => setMemberToRemove(user)}
                        >
                          Remove
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <TableCardEmpty>No members match your search.</TableCardEmpty>
            )}
          </TableCard>
          {members && members.data.length > 0 && (
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
        title={`Add member to ${team.name}`}
      >
        <AddMemberForm teamId={team.id} onDone={() => setIsAdding(false)} />
      </Modal>

      <ConfirmDialog
        isOpen={memberToRemove !== null}
        onClose={() => setMemberToRemove(null)}
        onConfirm={() =>
          memberToRemove &&
          removeMember.mutate({ teamId: team.id, userId: memberToRemove.id })
        }
        title="Remove Member"
        message={`Remove "${memberToRemove?.username}" from "${team.name}"? They will lose access to this team's sites straight away.`}
        confirmText="Remove"
        isDestructive
      />
    </>
  );
}

function AddMemberForm({
  teamId,
  onDone,
}: {
  teamId: number;
  onDone: () => void;
}) {
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const { data: candidates } = useAdminUsers({
    page: 1,
    perPage: 20,
    search: debouncedSearch || undefined,
    excludeTeamId: teamId,
    active: true,
  });
  const addMember = useAddTeamMember();

  const users = candidates?.data ?? [];
  const total = candidates?.total ?? 0;

  return (
    <div className="form-column">
      <label className="field-label">
        <span className="field-label-text">Find a user</span>
        <FormInput
          type="search"
          placeholder="Search users to add..."
          value={searchInput}
          onChange={(e) => setSearchInput(e.target.value)}
        />
      </label>
      {users.length > 0 ? (
        <ul className="member-candidates">
          {users.map((user) => (
            <li key={user.id}>
              <span>
                {user.username}
                <span className="muted-text"> ({user.role})</span>
              </span>
              <button
                type="button"
                className="button-table-action compact-input"
                aria-label={`Add ${user.username}`}
                onClick={() =>
                  addMember.mutate({ teamId, member: { user_id: user.id } })
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
            ? "No matching users."
            : "No other active users to add."}
        </p>
      )}
      {total > users.length && (
        <p className="muted-text" style={{ margin: 0, fontSize: "12px" }}>
          Showing {users.length} of {total} users. Refine your search.
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
