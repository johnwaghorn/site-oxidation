import { useState, useRef, useEffect, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import type { ThemePreference } from "../../hooks/useThemePreference";

interface UserMenuProps {
  username: string;
  isAdmin: boolean;
  themePreference: ThemePreference;
  variant?: "button" | "sidebar";
  showAdminLink?: boolean;
  onChangePassword: () => void;
  onLogout: () => void;
  onThemePreferenceChange: (preference: ThemePreference) => void;
}

const menuItemStyle: CSSProperties = {
  display: "block",
  width: "100%",
  padding: "10px 16px",
  textAlign: "left",
  background: "none",
  border: "none",
  borderBottom: "1px solid var(--color-border)",
  borderRadius: 0,
  color: "var(--color-text)",
  cursor: "pointer",
};

export function UserMenu({
  username,
  isAdmin,
  themePreference,
  variant = "button",
  showAdminLink = true,
  onChangePassword,
  onLogout,
  onThemePreferenceChange,
}: UserMenuProps) {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const isSidebar = variant === "sidebar";

  return (
    <div
      ref={menuRef}
      style={{ position: "relative", width: isSidebar ? "100%" : undefined }}
    >
      <button
        className={isSidebar ? "app-sidebar-account" : "button-user-menu"}
        onClick={() => setIsOpen(!isOpen)}
      >
        {isSidebar && (
          <span className="app-sidebar-account-avatar">
            {username.slice(0, 1).toUpperCase()}
          </span>
        )}
        <span>{username}</span>
        <span aria-hidden="true">▾</span>
      </button>
      {isOpen && (
        <div
          style={{
            position: "absolute",
            right: isSidebar ? "auto" : 0,
            left: isSidebar ? 0 : "auto",
            top: isSidebar ? "auto" : "100%",
            bottom: isSidebar ? "calc(100% + 8px)" : "auto",
            marginTop: isSidebar ? 0 : "4px",
            minWidth: "220px",
            zIndex: 10,
            backgroundColor: "var(--color-surface-elevated)",
            border: "1px solid var(--color-border)",
            borderRadius: "12px",
            boxShadow: "var(--shadow-popover)",
            overflow: "hidden",
          }}
        >
          <div
            style={{
              padding: "12px 16px",
              borderBottom: "1px solid var(--color-border)",
            }}
          >
            <label
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "6px",
                color: "var(--color-muted)",
                fontSize: "12px",
                fontWeight: 600,
              }}
            >
              Theme
              <select
                value={themePreference}
                onChange={(event) =>
                  onThemePreferenceChange(event.target.value as ThemePreference)
                }
                style={{
                  padding: "8px 10px",
                  border: "1px solid var(--color-border)",
                  borderRadius: "8px",
                  backgroundColor: "var(--color-field-bg)",
                  color: "var(--color-text)",
                  font: "inherit",
                }}
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </label>
          </div>
          {isAdmin && showAdminLink && (
            <button
              onClick={() => {
                setIsOpen(false);
                void navigate("/admin/teams");
              }}
              style={menuItemStyle}
            >
              Admin Panel
            </button>
          )}
          <button
            onClick={() => {
              setIsOpen(false);
              onChangePassword();
            }}
            style={menuItemStyle}
          >
            Change Password
          </button>
          <button
            onClick={() => {
              setIsOpen(false);
              onLogout();
            }}
            style={{ ...menuItemStyle, color: "var(--color-danger)" }}
          >
            Logout
          </button>
        </div>
      )}
    </div>
  );
}
