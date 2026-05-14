import { useState } from "react";
import { useBootstrap } from "../hooks/useSetup";
import { ErrorMessage } from "../components/ui/ErrorMessage";
import { pageWrapper, formInput, subtitle } from "../lib/styles";

interface SetupProps {
  onSetupComplete: () => void;
}

export function Setup({ onSetupComplete }: SetupProps) {
  const [generatedPassword, setGeneratedPassword] = useState<string | null>(
    null,
  );
  const bootstrap = useBootstrap();

  const handleBootstrap = () => {
    bootstrap.mutate(undefined, {
      onSuccess: (data) => {
        setGeneratedPassword(data.password);
      },
    });
  };

  return (
    <div style={pageWrapper}>
      <h1 style={{ marginBottom: "8px" }}>Welcome to Site Oxidation</h1>
      <p style={subtitle}>
        You are first. No admin account exists yet. Create one to get started.
      </p>
      {generatedPassword ? (
        <>
          <p>
            Save this password now. It is only shown once. Store in a password
            manager that encrypts your passwords if you can!
          </p>
          <pre>{generatedPassword}</pre>
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
