import { useEffect, useState, type FormEvent } from "react";
import { Link } from "react-router-dom";
import {
  useAdminUsers,
  useCreateUser,
  useUpdateUser,
  useResetPassword,
} from "../hooks/useAdmin";
import { usePagination } from "../hooks/usePagination";
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
  const [createError, setCreateError] = useState<string | null>(null);

  const [resettingUserId, setResettingUserId] = useState<number | null>(null);
  const [tempPassword, setTempPassword] = useState("");

  const handleCreateUser = (e: FormEvent) => {
    e.preventDefault();
    setCreateError(null);
    createUser.mutate(
      { username: newUsername.trim(), password: newPassword, role: newRole },
      {
        onSuccess: () => {
          setNewUsername("");
          setNewPassword("");
          setNewRole("user");
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
          error={createError}
          isPending={createUser.isPending}
          onUsernameChange={setNewUsername}
          onPasswordChange={setNewPassword}
          onRoleChange={setNewRole}
          onSubmit={handleCreateUser}
          onCancel={() => setShowCreateForm(false)}
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
  error: string | null;
  isPending: boolean;
  onUsernameChange: (v: string) => void;
  onPasswordChange: (v: string) => void;
  onRoleChange: (v: UserRole) => void;
  onSubmit: (e: FormEvent) => void;
  onCancel: () => void;
}

function CreateUserForm({
  username,
  password,
  role,
  error,
  isPending,
  onUsernameChange,
  onPasswordChange,
  onRoleChange,
  onSubmit,
  onCancel,
}: CreateUserFormProps) {
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
      {error && <p style={errorBox}>{error}</p>}
      <div style={{ display: "flex", gap: "8px" }}>
        <button type="submit" disabled={isPending} style={formInput}>
          {isPending ? "Creating..." : "Create User"}
        </button>
        <button type="button" onClick={onCancel} style={formInput}>
          Cancel
        </button>
      </div>
    </form>
  );
}
