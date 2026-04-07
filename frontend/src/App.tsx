import { useEffect, useState } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import type { components } from "./generated/schema";
import { api } from "./lib/api";
import { Dashboard } from "./pages/Dashboard";
import { SiteDetail } from "./pages/SiteDetail";
import { Login } from "./pages/Login";
import { Setup } from "./pages/Setup";
import { ChangePassword } from "./pages/ChangePassword";
import { useAuth } from "./hooks/useAuth";
import { LoadingSpinner } from "./components/ui/LoadingSpinner";
import { centeredFullScreen } from "./lib/styles";

type SetupStatus = components["schemas"]["SetupStatus"];

function App() {
  const {
    isAuthenticated,
    isLoading,
    logout,
    username,
    role,
    mustChangePassword,
    refresh,
  } = useAuth();
  const [setupRequired, setSetupRequired] = useState<boolean | null>(null);
  const [changingPassword, setChangingPassword] = useState(false);

  useEffect(() => {
    async function loeadSetupStatus() {
      try {
        const { data, response } = await api.GET("/api/setup/status");
        if (!response.ok) {
          setSetupRequired(false);
          return;
        }
        const status: SetupStatus = data ?? { setup_required: false };
        setSetupRequired(Boolean(status.setup_required));
      } catch {
        setSetupRequired(false);
      }
    }
    void loeadSetupStatus();
  }, []);

  if (isLoading || setupRequired === null) {
    return (
      <div style={centeredFullScreen}>
        <LoadingSpinner />
      </div>
    );
  }

  if (setupRequired) {
    return <Setup onSetupComplete={() => setSetupRequired(false)} />;
  }

  if (!isAuthenticated) {
    return <Login onLoginSuccess={refresh} />;
  }

  if (mustChangePassword || changingPassword) {
    return (
      <ChangePassword
        onPasswordChanged={() => {
          setChangingPassword(false);
          void refresh();
        }}
        onCancel={
          changingPassword ? () => setChangingPassword(false) : undefined
        }
      />
    );
  }

  return (
    <BrowserRouter>
      <Routes>
        <Route
          path="/"
          element={
            <Dashboard
              username={username}
              role={role}
              onLogout={logout}
              onChangePassword={() => setChangingPassword(true)}
            />
          }
        />
        <Route path="/sites/:id" element={<SiteDetail />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
