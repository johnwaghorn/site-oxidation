import { useEffect, useState } from "react";
import { BrowserRouter, Navigate, Routes, Route } from "react-router-dom";
import { Dashboard } from "./pages/Dashboard";
import { SiteDetail } from "./pages/SiteDetail";
import { Login } from "./pages/Login";
import { Setup } from "./pages/Setup";
import { ChangePassword, ChangePasswordForm } from "./pages/ChangePassword";
import { AdminTeams } from "./pages/AdminTeams";
import { AdminTeamDetail } from "./pages/AdminTeamDetail";
import { AdminTeamMembers } from "./pages/AdminTeamMembers";
import { AdminTeamSites } from "./pages/AdminTeamSites";
import { AdminUsers } from "./pages/AdminUsers";
import { AdminUserDetail } from "./pages/AdminUserDetail";
import { AdminCanary } from "./pages/AdminCanary";
import { Notifications } from "./pages/Notifications";
import { ImportExport } from "./pages/ImportExport";
import { NotFound } from "./pages/NotFound";
import { useAuth } from "./hooks/useAuth";
import { useSetupStatus } from "./hooks/useSetup";
import { useThemePreference } from "./hooks/useThemePreference";
import { LoadingSpinner } from "./components/ui/LoadingSpinner";
import { AppShell } from "./components/ui/AppShell";
import { Modal } from "./components/ui/Modal";

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
      <div className="centered-full-screen">
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

  if (mustChangePassword) {
    return <ChangePassword onPasswordChanged={() => void refresh()} />;
  }

  return (
    <BrowserRouter>
      <AppShell
        username={username ?? ""}
        role={role}
        themePreference={themePreference}
        onLogout={logout}
        onChangePassword={() => setChangingPassword(true)}
        onThemePreferenceChange={handleThemePreferenceChange}
      >
        <Routes>
          <Route path="/" element={<Dashboard role={role} teams={teams} />} />
          <Route path="/notifications" element={<Notifications />} />
          <Route path="/notifications/:teamId" element={<Notifications />} />
          <Route path="/import-export" element={<ImportExport />} />
          <Route path="/sites/:id" element={<SiteDetail />} />
          {role === "admin" && (
            <>
              <Route path="/admin/teams" element={<AdminTeams />} />
              <Route path="/admin/teams/:id" element={<AdminTeamDetail />}>
                <Route index element={<Navigate to="members" replace />} />
                <Route path="members" element={<AdminTeamMembers />} />
                <Route path="sites" element={<AdminTeamSites />} />
              </Route>
              <Route path="/admin/users" element={<AdminUsers />} />
              <Route path="/admin/users/:id" element={<AdminUserDetail />} />
              <Route path="/admin/canary" element={<AdminCanary />} />
            </>
          )}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </AppShell>
      <Modal
        isOpen={changingPassword}
        onClose={() => setChangingPassword(false)}
        title="Change password"
      >
        <ChangePasswordForm
          onPasswordChanged={() => {
            setChangingPassword(false);
            void refresh();
          }}
          onCancel={() => setChangingPassword(false)}
        />
      </Modal>
    </BrowserRouter>
  );
}

export default App;
