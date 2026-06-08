import { useState, useRef, useEffect, type CSSProperties } from "react";
import { useNavigate } from "react-router-dom";
import type { ThemePreference } from "../../hooks/useThemePreference";

interface UserMenuProps {
  username: string;
  isAdmin: boolean;
  themePreference: ThemePreference;
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

  return (
    <div ref={menuRef} style={{ position: "relative" }}>
      <button onClick={() => setIsOpen(!isOpen)}>{username} ▾</button>
      {isOpen && (
        <div
          style={{
            position: "absolute",
            right: 0,
            top: "100%",
            marginTop: "4px",
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
          {isAdmin && (
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
