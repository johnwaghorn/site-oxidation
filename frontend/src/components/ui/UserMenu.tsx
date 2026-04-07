import { useState, useRef, useEffect } from "react";

interface UserMenuProps {
  username: string;
  isAdmin: boolean;
  onChangePassword: () => void;
  onLogout: () => void;
}

const menuItemStyle: React.CSSProperties = {
  display: "block",
  width: "100%",
  padding: "10px 16px",
  textAlign: "left",
  background: "none",
  border: "none",
  borderBottom: "1px solid #333",
  borderRadius: 0,
  cursor: "pointer",
};

export function UserMenu({
  username,
  isAdmin,
  onChangePassword,
  onLogout,
}: UserMenuProps) {
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
            minWidth: "180px",
            zIndex: 10,
            backgroundColor: "#242424",
            border: "1px solid #333",
            borderRadius: "8px",
            overflow: "hidden",
          }}
        >
          {isAdmin && (
            <button
              onClick={() => {
                setIsOpen(false);
                window.location.href = "/admin/users";
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
            style={{ ...menuItemStyle, color: "#dc2626" }}
          >
            Logout
          </button>
        </div>
      )}
    </div>
  );
}
