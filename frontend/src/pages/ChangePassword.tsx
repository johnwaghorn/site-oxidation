import { useState, type FormEvent } from "react";
import { api } from "../lib/api";
import {
  formPageWrapper,
  formCard,
  formColumn,
  formInput,
  errorBox,
  subtitle,
} from "../lib/styles";
import { FormInput } from "../components/ui/FormControls";
import type { components } from "../generated/schema";

type ChangePasswordRequest = components["schemas"]["ChangePasswordRequest"];

interface ChangePasswordProps {
  onPasswordChanged: () => void;
  onCancel?: () => void;
}

export function ChangePassword({
  onPasswordChanged,
  onCancel,
}: ChangePasswordProps) {
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
    <div style={formPageWrapper}>
      <div style={formCard}>
        <h1 style={{ marginBottom: "8px" }}>Change Password</h1>
        <p style={subtitle}>
          {onCancel
            ? "Enter your current password and choose a new one."
            : "You must change your password before continuing."}
        </p>
        <form onSubmit={handleSubmit} style={formColumn}>
          <FormInput
            type="password"
            placeholder="Current password"
            value={currentPassword}
            onChange={(e) => setCurrentPassword(e.target.value)}
            required
            autoComplete="current-password"
          />
          <FormInput
            type="password"
            placeholder="New password (12+ characters)"
            value={newPassword}
            onChange={(e) => setNewPassword(e.target.value)}
            required
            minLength={12}
            autoComplete="new-password"
          />
          <FormInput
            type="password"
            placeholder="Confirm new password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            required
            minLength={12}
            autoComplete="new-password"
          />
          {error && <p style={errorBox}>{error}</p>}
          <button
            type="submit"
            disabled={isLoading}
            style={{ ...formInput, cursor: isLoading ? "wait" : "pointer" }}
          >
            {isLoading ? "Changing..." : "Change Password"}
          </button>
          {onCancel && (
            <button type="button" onClick={onCancel} style={formInput}>
              Cancel
            </button>
          )}
        </form>
      </div>
    </div>
  );
}
