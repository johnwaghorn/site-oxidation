import { useState } from "react";
import { useBootstrap } from "../hooks/useSetup";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { CopyButton } from "../components/ui/CopyButton";
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
    <div className="page-wrapper">
      <h1 style={{ marginBottom: "8px" }}>Welcome to Site Oxidation</h1>
      <p className="page-subtitle">
        You are first. No admin account exists yet. Create one to get started.
      </p>
      {newAdmin ? (
        <>
          <p>
            Save these now. The password is only shown once. Store them in a
            password manager that encrypts your passwords if you can!
          </p>
          <p style={{ marginBottom: "4px", fontWeight: 500 }}>Username</p>
          <Credential value={newAdmin.username} testId="generated-username" />
          <p style={{ marginBottom: "4px", fontWeight: 500 }}>Password</p>
          <Credential value={newAdmin.password} testId="generated-password" />
          <button onClick={onSetupComplete} className="form-input">
            I saved my password, continue
          </button>
        </>
      ) : (
        <button
          className="button-primary-action form-input"
          onClick={handleBootstrap}
          disabled={bootstrap.isPending}
        >
          {bootstrap.isPending ? "Creating admin..." : "Create admin user"}
        </button>
      )}
      {bootstrap.isError && <ErrorMessage error={bootstrap.error} />}
    </div>
  );
}

function Credential({ value, testId }: { value: string; testId: string }) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: "8px",
        marginBottom: "16px",
        maxWidth: "640px",
        padding: "8px",
        border: "1px solid var(--color-border)",
        borderRadius: "10px",
        backgroundColor: "var(--color-surface)",
      }}
    >
      <pre
        data-testid={testId}
        style={{
          margin: 0,
          padding: "4px 8px",
          overflowX: "auto",
          color: "var(--color-text)",
        }}
      >
        {value}
      </pre>
      <CopyButton value={value} />
    </div>
  );
}
