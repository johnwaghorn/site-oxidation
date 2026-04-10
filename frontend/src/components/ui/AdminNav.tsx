import { Link, useLocation } from "react-router-dom";
import type { CSSProperties } from "react";

const navStyle: CSSProperties = {
  display: "flex",
  gap: "0",
  marginBottom: "24px",
};

const tabBase: CSSProperties = {
  padding: "10px 20px",
  textDecoration: "none",
  borderBottom: "2px solid transparent",
  fontSize: "14px",
  fontWeight: 500,
};

const activeTab: CSSProperties = {
  ...tabBase,
  borderBottomColor: "#646cff",
  color: "#646cff",
};

const inactiveTab: CSSProperties = {
  ...tabBase,
  color: "#6b7280",
};

export function AdminNav() {
  const location = useLocation();

  return (
    <nav style={navStyle}>
      <Link
        to="/admin/teams"
        style={location.pathname === "/admin/teams" ? activeTab : inactiveTab}
      >
        Teams
      </Link>
      <Link
        to="/admin/users"
        style={location.pathname === "/admin/users" ? activeTab : inactiveTab}
      >
        Users
      </Link>
    </nav>
  );
}
