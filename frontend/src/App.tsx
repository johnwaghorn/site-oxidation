import { useEffect, useState } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { SiteDetail } from "./pages/SiteDetail";
import { Login } from "./pages/Login";
import { Setup } from "./pages/Setup";
import { ChangePassword } from "./pages/ChangePassword";
import { AdminTeams } from "./pages/AdminTeams";
import { AdminTeamDetail } from "./pages/AdminTeamDetail";
import { AdminUsers } from "./pages/AdminUsers";
import { useAuth } from "./hooks/useAuth";
import { useSetupStatus } from "./hooks/useSetup";
import { useThemePreference } from "./hooks/useThemePreference";
import { LoadingSpinner } from "./components/ui/LoadingSpinner";
import { centeredFullScreen } from "./lib/styles";

function App() {
  const { themePreference, setThemePreference } = useThemePreference();
  const {
    isAuthenticated,
    isLoading: authLoading,
    logout,
    username,
    role,
    teams,
    mustChangePassword,
    themePreference: savedThemePreference,
    refresh,
    updateThemePreference,
  } = useAuth();
  const {
    setupRequired,
    isLoading: setupLoading,
    refresh: refreshSetup,
  } = useSetupStatus();
  const [changingPassword, setChangingPassword] = useState(false);

  useEffect(() => {
    if (savedThemePreference && savedThemePreference !== themePreference) {
      setThemePreference(savedThemePreference);
    }
  }, [savedThemePreference, setThemePreference, themePreference]);

  const handleThemePreferenceChange = (preference: typeof themePreference) => {
    setThemePreference(preference);
    void updateThemePreference(preference);
  };

  if (authLoading || setupLoading) {
    return (
      <div style={centeredFullScreen}>
        <LoadingSpinner />
      </div>
    );
  }

  if (setupRequired) {
    return <Setup onSetupComplete={refreshSetup} />;
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
              teams={teams}
              themePreference={themePreference}
              onLogout={logout}
              onChangePassword={() => setChangingPassword(true)}
              onThemePreferenceChange={handleThemePreferenceChange}
            />
          }
        />
        <Route path="/sites/:id" element={<SiteDetail />} />
        {role === "admin" && (
          <>
            <Route path="/admin/teams" element={<AdminTeams />} />
            <Route path="/admin/teams/:id" element={<AdminTeamDetail />} />
            <Route path="/admin/users" element={<AdminUsers />} />
          </>
        )}
      </Routes>
    </BrowserRouter>
  );
}

export default App;
