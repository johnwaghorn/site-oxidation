import type { CSSProperties } from "react";

export const backLink: CSSProperties = {
  color: "var(--color-muted)",
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
  borderBottomColor: "var(--color-primary)",
  color: "var(--color-primary)",
};

export const inactiveTab: CSSProperties = {
  ...tabBase,
  color: "var(--color-muted)",
};
