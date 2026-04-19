import type { CSSProperties } from "react";

export const backLink: CSSProperties = {
  color: "#6b7280",
  textDecoration: "none",
  marginBottom: "16px",
  display: "inline-block",
};

export const tabsNav: CSSProperties = {
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

export const activeTab: CSSProperties = {
  ...tabBase,
  borderBottomColor: "#646cff",
  color: "#646cff",
};

export const inactiveTab: CSSProperties = {
  ...tabBase,
  color: "#6b7280",
};
