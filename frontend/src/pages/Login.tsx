import { useState, type FormEvent } from "react";
import { api } from "../lib/api";
import { FormInput, SecretInput } from "../components/ui/FormControls";
import type { components } from "../generated/schema";
import { usePageTitle } from "../hooks/usePageTitle";

type Credentials = components["schemas"]["Credentials"];

interface LoginProps {
  onLoginSuccess: () => void;
}

export function Login({ onLoginSuccess }: LoginProps) {
  usePageTitle("Log in");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    setError("");
    try {
      const credentials: Credentials = { username, password };
      const { error } = await api.POST("/api/auth/login", {
        body: credentials,
      });
      if (!error) {
        onLoginSuccess();
      } else {
        setError(
          error.error === "unauthorized"
            ? "Incorrect username or password. Please try again."
            : error.message,
        );
      }
    } catch {
      setError("Login failed. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="form-page-wrapper">
      <div className="form-card">
        <h1 style={{ marginBottom: "24px" }}>Site Oxidation</h1>
        <form onSubmit={handleSubmit} className="form-column">
          <label className="field-label">
            <span className="field-label-text">Username</span>
            <FormInput
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
              autoComplete="username"
            />
          </label>
          <label className="field-label">
            <span className="field-label-text">Password</span>
            <SecretInput
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              autoComplete="current-password"
            />
          </label>
          {error && (
            <p className="error-box" role="alert">
              {error}
            </p>
          )}
          <button
            type="submit"
            disabled={isLoading}
            className="form-input"
            style={{ cursor: isLoading ? "wait" : "pointer" }}
          >
            {isLoading ? "Logging in..." : "Login"}
          </button>
        </form>
      </div>
    </div>
  );
}
