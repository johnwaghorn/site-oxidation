import { useState, type FormEvent } from "react";
import { api } from "../lib/api";
import { SecretInput } from "../components/ui/FormControls";
import type { components } from "../generated/schema";

type ChangePasswordRequest = components["schemas"]["ChangePasswordRequest"];

interface ChangePasswordFormProps {
  onPasswordChanged: () => void;
  onCancel?: () => void;
}

export function ChangePassword({
  onPasswordChanged,
}: {
  onPasswordChanged: () => void;
}) {
  return (
    <div className="form-page-wrapper">
      <div className="form-card">
        <h1 style={{ marginBottom: "8px" }}>Change Password</h1>
        <p className="page-subtitle">
          You must change your password before continuing.
        </p>
        <ChangePasswordForm onPasswordChanged={onPasswordChanged} />
      </div>
    </div>
  );
}

export function ChangePasswordForm({
  onPasswordChanged,
  onCancel,
}: ChangePasswordFormProps) {
  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setError(null);
    if (newPassword !== confirmPassword) {
      setError("New passwords do not match");
      return;
    }
    const payload: ChangePasswordRequest = {
      current_password: currentPassword,
      new_password: newPassword,
    };
    setIsLoading(true);
    try {
      const { error: apiError } = await api.POST("/api/auth/change-password", {
        body: payload,
      });
      if (apiError) {
        setError(apiError.message ?? "Failed to change password");
        return;
      }
      onPasswordChanged();
    } catch {
      setError("Failed to change password");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="form-column">
      <SecretInput
        placeholder="Current password"
        value={currentPassword}
        onChange={(e) => setCurrentPassword(e.target.value)}
        required
        autoComplete="current-password"
      />
      <SecretInput
        placeholder="New password (12+ characters)"
        value={newPassword}
        onChange={(e) => setNewPassword(e.target.value)}
        required
        minLength={12}
        autoComplete="new-password"
      />
      <SecretInput
        placeholder="Confirm new password"
        value={confirmPassword}
        onChange={(e) => setConfirmPassword(e.target.value)}
        required
        minLength={12}
        autoComplete="new-password"
      />
      {error && <p className="error-box">{error}</p>}
      <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        {onCancel && (
          <button
            type="button"
            className="button-secondary-action"
            onClick={onCancel}
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          className="button-primary-action"
          disabled={isLoading}
        >
          {isLoading ? "Changing..." : "Change Password"}
        </button>
      </div>
    </form>
  );
}
