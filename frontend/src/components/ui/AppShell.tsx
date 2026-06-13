import { NavLink } from "react-router-dom";
import type { ReactNode } from "react";
import { UserMenu } from "./UserMenu";
import type { ThemePreference } from "../../hooks/useThemePreference";
import type { components } from "../../generated/schema";

type UserRole = components["schemas"]["UserRole"];

interface AppShellProps {
  children: ReactNode;
  username: string;
  role: UserRole | null;
  themePreference: ThemePreference;
  onChangePassword: () => void;
  onLogout: () => void;
  onThemePreferenceChange: (preference: ThemePreference) => void;
}

export function AppShell({
  children,
  username,
  role,
  themePreference,
  onChangePassword,
  onLogout,
  onThemePreferenceChange,
}: AppShellProps) {
  const isAdmin = role === "admin";

  return (
    <div className="app-shell">
      <aside className="app-sidebar" aria-label="Primary navigation">
        <div>
          <div className="app-sidebar-brand">
            <img
              className="app-sidebar-logo"
              src="/site-oxidation.svg"
              alt=""
              aria-hidden="true"
            />
            <div>
              <div className="app-sidebar-title">Site Oxidation</div>
              <div className="app-sidebar-subtitle">Monitoring</div>
            </div>
          </div>

          <nav className="app-sidebar-nav">
            <NavLink
              to="/"
              end
              className={({ isActive }) =>
                `app-sidebar-link${isActive ? " active" : ""}`
              }
            >
              Sites
            </NavLink>
            <NavLink
              to="/notifications"
              className={({ isActive }) =>
                `app-sidebar-link${isActive ? " active" : ""}`
              }
            >
              Notifications
            </NavLink>
            <NavLink
              to="/backups"
              className={({ isActive }) =>
                `app-sidebar-link${isActive ? " active" : ""}`
              }
            >
              Backups & Restores
            </NavLink>

            {isAdmin && (
              <div className="app-sidebar-section">
                <div className="app-sidebar-section-label">Admin</div>
                <NavLink
                  to="/admin/teams"
                  className={({ isActive }) =>
                    `app-sidebar-link${isActive ? " active" : ""}`
                  }
                >
                  Teams
                </NavLink>
                <NavLink
                  to="/admin/users"
                  className={({ isActive }) =>
                    `app-sidebar-link${isActive ? " active" : ""}`
                  }
                >
                  Users
                </NavLink>
              </div>
            )}
          </nav>
        </div>

        <UserMenu
          username={username}
          isAdmin={isAdmin}
          themePreference={themePreference}
          variant="sidebar"
          showAdminLink={false}
          onChangePassword={onChangePassword}
          onLogout={onLogout}
          onThemePreferenceChange={onThemePreferenceChange}
        />
      </aside>

      <main className="app-shell-main">{children}</main>
    </div>
  );
}
