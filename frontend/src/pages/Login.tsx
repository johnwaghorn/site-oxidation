import { useState, type FormEvent } from "react";
import { api } from "../lib/api";
import { FormInput } from "../components/ui/FormControls";
import type { components } from "../generated/schema";

type Credentials = components["schemas"]["Credentials"];

interface LoginProps {
  onLoginSuccess: () => void;
}

export function Login({ onLoginSuccess }: LoginProps) {
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
          error.error === "too_many_requests"
            ? error.message
            : "Incorrect username or password. Please try again.",
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
          <FormInput
            type="text"
            placeholder="Username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            autoComplete="username"
          />
          <FormInput
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            autoComplete="current-password"
          />
          {error && <p className="error-box">{error}</p>}
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
