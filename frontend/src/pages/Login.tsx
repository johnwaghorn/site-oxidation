import { useState, type FormEvent } from "react";
import { api } from "../lib/api";
import {
  formPageWrapper,
  formColumn,
  formInput,
  errorBox,
} from "../lib/styles";
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
    <div style={formPageWrapper}>
      <h1 style={{ marginBottom: "24px" }}>Site Oxidation</h1>
      <form onSubmit={handleSubmit} style={formColumn}>
        <input
          type="text"
          placeholder="Username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          required
          autoComplete="username"
          style={formInput}
        />
        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
          autoComplete="current-password"
          style={formInput}
        />
        {error && <p style={errorBox}>{error}</p>}
        <button
          type="submit"
          disabled={isLoading}
          style={{ ...formInput, cursor: isLoading ? "wait" : "pointer" }}
        >
          {isLoading ? "Logging in..." : "Login"}
        </button>
      </form>
    </div>
  );
}
