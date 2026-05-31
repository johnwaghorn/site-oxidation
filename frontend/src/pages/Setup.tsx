import { useState } from "react";
import { useBootstrap } from "../hooks/useSetup";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { pageWrapper, formInput, subtitle } from "../lib/styles";
import type { components } from "../generated/schema";

type BootstrapResponse = components["schemas"]["BootstrapResponse"];

interface SetupProps {
  onSetupComplete: () => void;
}

export function Setup({ onSetupComplete }: SetupProps) {
  const [newAdmin, setNewAdmin] = useState<BootstrapResponse | null>(null);
  const bootstrap = useBootstrap();

  const handleBootstrap = () => {
    bootstrap.mutate(undefined, {
      onSuccess: (data) => {
        setNewAdmin(data);
      },
    });
  };

  return (
    <div style={pageWrapper}>
      <h1 style={{ marginBottom: "8px" }}>Welcome to Site Oxidation</h1>
      <p style={subtitle}>
        You are first. No admin account exists yet. Create one to get started.
      </p>
      {newAdmin ? (
        <>
          <p>
            Save these now. The password is only shown once. Store them in a
            password manager that encrypts your passwords if you can!
          </p>
          <p style={{ marginBottom: "4px", fontWeight: 500 }}>Username</p>
          <pre>{newAdmin.username}</pre>
          <p style={{ marginBottom: "4px", fontWeight: 500 }}>Password</p>
          <pre>{newAdmin.password}</pre>
          <button onClick={onSetupComplete} style={formInput}>
            I saved my password, continue
          </button>
        </>
      ) : (
        <button
          onClick={handleBootstrap}
          disabled={bootstrap.isPending}
          style={formInput}
        >
          {bootstrap.isPending ? "Creating admin..." : "Create admin user"}
        </button>
      )}
      {bootstrap.isError && <ErrorMessage error={bootstrap.error} />}
    </div>
  );
}
