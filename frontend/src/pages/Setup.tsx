import { useState } from "react";
import { api } from "../lib/api";
import { pageWrapper, errorBox, formInput, subtitle } from "../lib/styles";
import type { components } from "../generated/schema";

type SetupStatus = components["schemas"]["SetupStatus"];
type BootstrapResponse = components["schemas"]["BootstrapResponse"];

interface SetupProps {
  onSetupComplete: () => void;
}

export function Setup({ onSetupComplete }: SetupProps) {
  const [generatedPassword, setGeneratedPassword] = useState<string | null>(
    null,
  );
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleBootstrap() {
    setIsLoading(true);
    setError(null);
    try {
      const {
        data,
        error: apiError,
        response,
      } = await api.POST("/api/setup/bootstrap");
      if (!response.ok || apiError) {
        setError(apiError?.message ?? "Failed to bootstrap admin user");
        return;
      }
      const payload: BootstrapResponse | null = data ?? null;
      if (!payload?.password) {
        setError("Bootstrap succeeded but no password was returned");
        return;
      }
      setGeneratedPassword(payload.password);
    } catch {
      setError("Failed to bootstrap admin user");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleContinue() {
    const { data, response } = await api.GET("/api/setup/status");
    if (!response.ok) {
      onSetupComplete();
      return;
    }
    const status: SetupStatus = data ?? { setup_required: false };
    if (!status.setup_required) onSetupComplete();
  }
  return (
    <div style={pageWrapper}>
      <h1 style={{ marginBottom: "8px" }}>Welcome to Site Oxidation</h1>
      <p style={subtitle}>
        You are first. No admin account exists yet. Create one to get started.
      </p>
      {generatedPassword ? (
        <>
          <p>
            Save this password now. It is only shown once. Use a password
            manager that encrypts your passwords if you can!
          </p>
          <pre>{generatedPassword}</pre>
          <button onClick={handleContinue} style={formInput}>
            I saved my password, continue
          </button>
        </>
      ) : (
        <button
          onClick={handleBootstrap}
          disabled={isLoading}
          style={formInput}
        >
          {isLoading ? "Creating admin..." : "Create admin user"}
        </button>
      )}
      {error && <p style={errorBox}>{error}</p>}
    </div>
  );
}
