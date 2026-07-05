import {
  useEffect,
  useId,
  useState,
  type FormEvent,
  type KeyboardEvent,
} from "react";
import { Link } from "react-router-dom";
import {
  useAdminUsers,
  useTeamOptions,
  useCreateUser,
  useUpdateUser,
  useDeleteUser,
  useResetPassword,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { Pagination } from "../components/ui/Pagination";
import { ConfirmDialog } from "../components/ui/ConfirmDialog";
import { SearchInput, SearchToolbar } from "../components/ui/SearchInput";
import { FormInput, FormSelect } from "../components/ui/FormControls";
import { FormToggleButton } from "../components/ui/FormToggleButton";
import { Truncate } from "../components/ui/Truncate";
import type { components } from "../generated/schema";

type UserResponse = components["schemas"]["UserResponse"];
type UserRole = components["schemas"]["UserRole"];
type TeamOption = components["schemas"]["TeamOption"];

export function AdminUsers() {
  const { page, perPage, goToPage, resetPage } = usePagination();
  const [searchInput, setSearchInput] = useState("");
  const debouncedSearch = useDebouncedValue(searchInput.trim());
  const {
    data: users,
    isLoading,
    error,
  } = useAdminUsers({
    page,
    perPage,
    search: debouncedSearch || undefined,
  });
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const deleteUser = useDeleteUser();
  const resetPassword = useResetPassword();

  const totalPages = users ? Math.ceil(users.total / users.per_page) : 0;
  const hasNoUsers = users != null && users.total === 0 && !debouncedSearch;

  useEffect(() => {
    if (users && users.data.length === 0 && users.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [users, page, goToPage]);

  useEffect(() => {
    resetPage();
  }, [debouncedSearch, resetPage]);

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState<UserRole>("user");
  const [newTeamId, setNewTeamId] = useState<number | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);

  const [resettingUserId, setResettingUserId] = useState<number | null>(null);
  const [tempPassword, setTempPassword] = useState("");
  const [userToDelete, setUserToDelete] = useState<UserResponse | null>(null);

  const resetCreateForm = () => {
    setNewUsername("");
    setNewPassword("");
    setNewRole("user");
    setNewTeamId(null);
    setCreateError(null);
  };

  // Admins have no team, so clear any stale selection when leaving the user role -
  // otherwise the combobox unmounts blank while teamId lingers and gets submitted.
  const handleRoleChange = (role: UserRole) => {
    setNewRole(role);
    if (role !== "user") {
      setNewTeamId(null);
    }
  };

  const handleCreateUser = (e: FormEvent) => {
    e.preventDefault();
    setCreateError(null);
    createUser.mutate(
      {
        username: newUsername.trim(),
        password: newPassword,
        role: newRole,
        team_id: newRole === "user" ? newTeamId : null,
      },
      {
        onSuccess: () => {
          resetCreateForm();
          setShowCreateForm(false);
        },
        onError: (err) => {
          setCreateError(err.message);
        },
      },
    );
  };

  const handleToggleActive = (user: UserResponse) => {
    updateUser.mutate({
      id: user.id,
      user: { role: user.role, active: !user.active },
    });
  };

  const handleResetPassword = (userId: number) => {
    if (!tempPassword.trim()) return;
    resetPassword.mutate(
      { id: userId, payload: { temp_password: tempPassword } },
      {
        onSuccess: () => {
          setResettingUserId(null);
          setTempPassword("");
        },
      },
    );
  };

  return (
    <div className="page-wrapper">
      <h1 className="page-title">Users</h1>

      {deleteUser.isError && <ErrorMessage error={deleteUser.error} />}

      <SearchToolbar
        action={
          <FormToggleButton
            isOpen={showCreateForm}
            openLabel="Create User"
            onClick={() => {
              if (showCreateForm) {
                resetCreateForm();
              }
              setShowCreateForm(!showCreateForm);
            }}
          />
        }
      >
        {!hasNoUsers && (
          <SearchInput
            value={searchInput}
            onChange={setSearchInput}
            placeholder="Search users..."
          />
        )}
      </SearchToolbar>

      {showCreateForm && (
        <div style={{ marginBottom: "24px" }}>
          <CreateUserForm
            username={newUsername}
            password={newPassword}
            role={newRole}
            teamId={newTeamId}
            error={createError}
            isPending={createUser.isPending}
            onUsernameChange={setNewUsername}
            onPasswordChange={setNewPassword}
            onRoleChange={handleRoleChange}
            onTeamChange={setNewTeamId}
            onSubmit={handleCreateUser}
            onCancel={() => {
              resetCreateForm();
              setShowCreateForm(false);
            }}
          />
        </div>
      )}

      <div>
        {isLoading ? (
          <LoadingSpinner />
        ) : error ? (
          <ErrorMessage error={error} />
        ) : users && users.data.length > 0 ? (
          <>
            <table className="data-table">
              <thead>
                <tr className="table-header-row">
                  <th className="table-cell-left">Username</th>
                  <th className="table-cell-center">Role</th>
                  <th className="table-cell-center">Active</th>
                  <th className="table-cell-left">Teams</th>
                  <th className="table-cell-left">Actions</th>
                </tr>
              </thead>
              <tbody>
                {users.data.map((user) => (
                  <tr key={user.id} className="table-row">
                    <td className="table-cell-left" style={{ fontWeight: 500 }}>
                      {user.username}
                      {user.must_change_password && (
                        <span
                          className="muted-text"
                          style={{
                            fontSize: "12px",
                            marginLeft: "8px",
                          }}
                        >
                          (must change password)
                        </span>
                      )}
                    </td>
                    <td className="table-cell-center">{user.role}</td>
                    <td className="table-cell-center">
                      {user.active ? "Yes" : "No"}
                    </td>
                    <td
                      className="table-cell-left muted-text"
                      style={{
                        fontSize: "14px",
                      }}
                    >
                      <Truncate
                        text={user.team_names || "None"}
                        maxWidth={260}
                      />
                    </td>
                    <td className="table-cell">
                      <div
                        style={{
                          display: "flex",
                          gap: "8px",
                          flexDirection: "column",
                        }}
                      >
                        <div style={{ display: "flex", gap: "8px" }}>
                          <button
                            className="button-table-action compact-input"
                            onClick={() => handleToggleActive(user)}
                          >
                            {user.active ? "Deactivate" : "Activate"}
                          </button>
                          <button
                            className="button-table-action compact-input"
                            onClick={() => {
                              setResettingUserId(
                                resettingUserId === user.id ? null : user.id,
                              );
                              setTempPassword("");
                            }}
                          >
                            {resettingUserId === user.id
                              ? "Cancel"
                              : "Reset Password"}
                          </button>
                          <button
                            className="button-table-action button-table-danger compact-input"
                            onClick={() => setUserToDelete(user)}
                          >
                            Delete
                          </button>
                        </div>
                        {resettingUserId === user.id && (
                          <div style={{ display: "flex", gap: "8px" }}>
                            <FormInput
                              type="text"
                              placeholder="Temporary password (12+ chars)"
                              value={tempPassword}
                              onChange={(e) => setTempPassword(e.target.value)}
                              minLength={12}
                              className="compact-input"
                              style={{ flex: 1 }}
                            />
                            <button
                              className="button-table-action compact-input"
                              onClick={() => handleResetPassword(user.id)}
                              disabled={resetPassword.isPending}
                            >
                              Set
                            </button>
                          </div>
                        )}
                      </div>
                      {updateUser.isError && (
                        <p
                          className="error-box"
                          style={{
                            marginTop: "4px",
                            fontSize: "12px",
                          }}
                        >
                          {updateUser.error.message}
                        </p>
                      )}
                      {resetPassword.isError && resettingUserId === user.id && (
                        <p
                          className="error-box"
                          style={{
                            marginTop: "4px",
                            fontSize: "12px",
                          }}
                        >
                          {resetPassword.error.message}
                        </p>
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
          <p className="muted-text">No users found.</p>
        )}
      </div>
      <ConfirmDialog
        isOpen={userToDelete !== null}
        onClose={() => setUserToDelete(null)}
        onConfirm={() => userToDelete && deleteUser.mutate(userToDelete.id)}
        title="Delete User"
        message={`Are you sure you want to delete "${userToDelete?.username}"?`}
        confirmText="Delete"
        isDestructive
      />
    </div>
  );
}

interface CreateUserFormProps {
  username: string;
  password: string;
  role: UserRole;
  teamId: number | null;
  error: string | null;
  isPending: boolean;
  onUsernameChange: (v: string) => void;
  onPasswordChange: (v: string) => void;
  onRoleChange: (v: UserRole) => void;
  onTeamChange: (v: number | null) => void;
  onSubmit: (e: FormEvent) => void;
  onCancel: () => void;
}

function CreateUserForm({
  username,
  password,
  role,
  teamId,
  error,
  isPending,
  onUsernameChange,
  onPasswordChange,
  onRoleChange,
  onTeamChange,
  onSubmit,
  onCancel,
}: CreateUserFormProps) {
  const needsTeam = role === "user";
  return (
    <form
      onSubmit={onSubmit}
      className="form-column"
      style={{ maxWidth: "400px" }}
    >
      <label>
        <span className="muted-text">Username</span>
        <FormInput
          type="text"
          placeholder="e.g. jsmith"
          value={username}
          onChange={(e) => onUsernameChange(e.target.value)}
          required
          style={{ display: "block", width: "100%" }}
        />
      </label>
      <label>
        <span className="muted-text">Temporary password</span>
        <FormInput
          type="text"
          placeholder="12+ characters"
          value={password}
          onChange={(e) => onPasswordChange(e.target.value)}
          required
          minLength={12}
          style={{ display: "block", width: "100%" }}
        />
      </label>
      <label>
        <span className="muted-text">Role</span>
        <FormSelect
          value={role}
          onChange={(e) => onRoleChange(e.target.value as UserRole)}
          style={{ display: "block", width: "100%" }}
        >
          <option value="user">User</option>
          <option value="admin">Admin</option>
        </FormSelect>
      </label>
      {needsTeam && <TeamCombobox teamId={teamId} onChange={onTeamChange} />}
      {error && <p className="error-box">{error}</p>}
      <div style={{ display: "flex", gap: "8px" }}>
        <button
          type="submit"
          className="button-primary-action form-input"
          disabled={isPending || (needsTeam && teamId === null)}
        >
          {isPending ? "Creating..." : "Create User"}
        </button>
        <button
          type="button"
          className="button-secondary-action form-input"
          onClick={onCancel}
        >
          Cancel
        </button>
      </div>
    </form>
  );
}

function TeamCombobox({
  teamId,
  onChange,
}: {
  teamId: number | null;
  onChange: (id: number | null) => void;
}) {
  const listboxId = useId();
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const debounced = useDebouncedValue(query, 250);
  const { data: options, error } = useTeamOptions(debounced);

  const opts = options ?? [];
  const listOpen = open && opts.length > 0;
  const noTeams = debounced === "" && options?.length === 0;
  const optionId = (id: number) => `${listboxId}-opt-${id}`;
  const activeOption = activeIndex >= 0 ? opts[activeIndex] : undefined;

  const selectOption = (team: TeamOption) => {
    onChange(team.id);
    setQuery(team.name);
    setOpen(false);
    setActiveIndex(-1);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        if (!listOpen) {
          setOpen(true);
          return;
        }
        setActiveIndex((i) => Math.min(i + 1, opts.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setActiveIndex((i) => Math.max(i - 1, 0));
        break;
      case "Enter":
        if (activeOption) {
          e.preventDefault();
          selectOption(activeOption);
        }
        break;
      case "Escape":
        setOpen(false);
        break;
    }
  };

  return (
    <label>
      <span className="muted-text">Team</span>
      <div style={{ position: "relative" }}>
        <FormInput
          type="text"
          role="combobox"
          aria-expanded={listOpen}
          aria-controls={listboxId}
          aria-autocomplete="list"
          aria-activedescendant={
            activeOption ? optionId(activeOption.id) : undefined
          }
          placeholder="Search teams..."
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            onChange(null);
            setActiveIndex(-1);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          onBlur={() => setOpen(false)}
          onKeyDown={handleKeyDown}
          style={{ display: "block", width: "100%" }}
        />
        {listOpen && (
          <ul
            id={listboxId}
            role="listbox"
            aria-label="Teams"
            className="combobox-list"
          >
            {opts.map((t, i) => (
              <li
                key={t.id}
                id={optionId(t.id)}
                role="option"
                aria-selected={i === activeIndex}
                onMouseDown={(e) => {
                  e.preventDefault();
                  selectOption(t);
                }}
                onMouseEnter={() => setActiveIndex(i)}
                title={t.name}
                className={
                  i === activeIndex ? "combobox-item active" : "combobox-item"
                }
              >
                {t.name}
              </li>
            ))}
          </ul>
        )}
      </div>
      {teamId !== null && (
        <span className="muted-text" style={{ fontSize: "12px" }}>
          Team selected
        </span>
      )}
      {error && <p className="error-box">Couldn't load teams - try again.</p>}
      {noTeams && (
        <p className="muted-text">
          No teams yet - <Link to="/admin/teams">create a team</Link> first.
        </p>
      )}
    </label>
  );
}
