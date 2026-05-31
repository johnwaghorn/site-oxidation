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
  useResetPassword,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { AdminNav } from "../components/ui/AdminNav";
import { LoadingSpinner } from "../components/ui/LoadingSpinner";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { Pagination } from "../components/ui/Pagination";
import {
  pageWrapper,
  backLink,
  table,
  tableHeaderRow,
  tableRow,
  tableCellLeft,
  tableCellCenter,
  tableCell,
  compactInput,
  mutedText,
  formColumn,
  formInput,
  errorBox,
  comboboxList,
  comboboxItem,
  comboboxItemHovered,
} from "../lib/styles";
import type { components } from "../generated/schema";

type UserResponse = components["schemas"]["UserResponse"];
type UserRole = components["schemas"]["UserRole"];

export function AdminUsers() {
  const { page, perPage, goToPage } = usePagination();
  const { data: users, isLoading, error } = useAdminUsers({ page, perPage });
  const createUser = useCreateUser();
  const updateUser = useUpdateUser();
  const resetPassword = useResetPassword();

  const totalPages = users ? Math.ceil(users.total / users.per_page) : 0;

  useEffect(() => {
    if (users && users.data.length === 0 && users.total > 0 && page > 1) {
      goToPage(1);
    }
  }, [users, page, goToPage]);

  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newUsername, setNewUsername] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newRole, setNewRole] = useState<UserRole>("user");
  const [newTeamId, setNewTeamId] = useState<number | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);

  const [resettingUserId, setResettingUserId] = useState<number | null>(null);
  const [tempPassword, setTempPassword] = useState("");

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
    <div style={pageWrapper}>
      <Link to="/" style={backLink}>
        &larr; Back to Dashboard
      </Link>
      <AdminNav />

      {showCreateForm ? (
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
      ) : (
        <button onClick={() => setShowCreateForm(true)} style={compactInput}>
          Create User
        </button>
      )}

      <div style={{ marginTop: "24px" }}>
        {isLoading ? (
          <LoadingSpinner />
        ) : error ? (
          <ErrorMessage error={error} />
        ) : users && users.data.length > 0 ? (
          <>
            <table style={table}>
              <thead>
                <tr style={tableHeaderRow}>
                  <th style={tableCellLeft}>Username</th>
                  <th style={tableCellCenter}>Role</th>
                  <th style={tableCellCenter}>Active</th>
                  <th style={tableCellLeft}>Teams</th>
                  <th style={tableCellLeft}>Actions</th>
                </tr>
              </thead>
              <tbody>
                {users.data.map((user) => (
                  <tr key={user.id} style={tableRow}>
                    <td style={{ ...tableCellLeft, fontWeight: 500 }}>
                      {user.username}
                      {user.must_change_password && (
                        <span
                          style={{
                            ...mutedText,
                            fontSize: "12px",
                            marginLeft: "8px",
                          }}
                        >
                          (must change password)
                        </span>
                      )}
                    </td>
                    <td style={tableCellCenter}>{user.role}</td>
                    <td style={tableCellCenter}>
                      {user.active ? "Yes" : "No"}
                    </td>
                    <td
                      style={{
                        ...tableCellLeft,
                        ...mutedText,
                        fontSize: "14px",
                      }}
                    >
                      {user.team_names || "None"}
                    </td>
                    <td style={tableCell}>
                      <div
                        style={{
                          display: "flex",
                          gap: "8px",
                          flexDirection: "column",
                        }}
                      >
                        <div style={{ display: "flex", gap: "8px" }}>
                          <button
                            onClick={() => handleToggleActive(user)}
                            style={compactInput}
                          >
                            {user.active ? "Deactivate" : "Activate"}
                          </button>
                          <button
                            onClick={() => {
                              setResettingUserId(
                                resettingUserId === user.id ? null : user.id,
                              );
                              setTempPassword("");
                            }}
                            style={compactInput}
                          >
                            {resettingUserId === user.id
                              ? "Cancel"
                              : "Reset Password"}
                          </button>
                        </div>
                        {resettingUserId === user.id && (
                          <div style={{ display: "flex", gap: "8px" }}>
                            <input
                              type="text"
                              placeholder="Temporary password (12+ chars)"
                              value={tempPassword}
                              onChange={(e) => setTempPassword(e.target.value)}
                              minLength={12}
                              style={{ ...compactInput, flex: 1 }}
                            />
                            <button
                              onClick={() => handleResetPassword(user.id)}
                              disabled={resetPassword.isPending}
                              style={compactInput}
                            >
                              Set
                            </button>
                          </div>
                        )}
                      </div>
                      {updateUser.isError && (
                        <p
                          style={{
                            ...errorBox,
                            marginTop: "4px",
                            fontSize: "12px",
                          }}
                        >
                          {updateUser.error.message}
                        </p>
                      )}
                      {resetPassword.isError && resettingUserId === user.id && (
                        <p
                          style={{
                            ...errorBox,
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
          <p style={mutedText}>No users found.</p>
        )}
      </div>
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
    <form onSubmit={onSubmit} style={{ ...formColumn, maxWidth: "400px" }}>
      <label>
        <span style={mutedText}>Username</span>
        <input
          type="text"
          placeholder="e.g. jsmith"
          value={username}
          onChange={(e) => onUsernameChange(e.target.value)}
          required
          style={{ ...formInput, display: "block", width: "100%" }}
        />
      </label>
      <label>
        <span style={mutedText}>Temporary password</span>
        <input
          type="text"
          placeholder="12+ characters"
          value={password}
          onChange={(e) => onPasswordChange(e.target.value)}
          required
          minLength={12}
          style={{ ...formInput, display: "block", width: "100%" }}
        />
      </label>
      <label>
        <span style={mutedText}>Role</span>
        <select
          value={role}
          onChange={(e) => onRoleChange(e.target.value as UserRole)}
          style={{ ...formInput, display: "block", width: "100%" }}
        >
          <option value="user">User</option>
          <option value="admin">Admin</option>
        </select>
      </label>
      {needsTeam && <TeamCombobox teamId={teamId} onChange={onTeamChange} />}
      {error && <p style={errorBox}>{error}</p>}
      <div style={{ display: "flex", gap: "8px" }}>
        <button
          type="submit"
          disabled={isPending || (needsTeam && teamId === null)}
          style={formInput}
        >
          {isPending ? "Creating..." : "Create User"}
        </button>
        <button type="button" onClick={onCancel} style={formInput}>
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

  const selectOption = (team: { id: number; name: string }) => {
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
      <span style={mutedText}>Team</span>
      <div style={{ position: "relative" }}>
        <input
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
          style={{ ...formInput, display: "block", width: "100%" }}
        />
        {listOpen && (
          <ul
            id={listboxId}
            role="listbox"
            aria-label="Teams"
            style={comboboxList}
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
                style={{
                  ...comboboxItem,
                  ...(i === activeIndex ? comboboxItemHovered : null),
                }}
              >
                {t.name}
              </li>
            ))}
          </ul>
        )}
      </div>
      {teamId !== null && (
        <span style={{ ...mutedText, fontSize: "12px" }}>Team selected</span>
      )}
      {error && <p style={errorBox}>Couldn't load teams - try again.</p>}
      {noTeams && (
        <p style={mutedText}>
          No teams yet - <Link to="/admin/teams">create a team</Link> first.
        </p>
      )}
    </label>
  );
}
