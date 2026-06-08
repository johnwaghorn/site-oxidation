import type { CSSProperties } from "react";

export const pageWrapper: CSSProperties = {
  maxWidth: "1200px",
  margin: "0 auto",
  padding: "28px 24px",
};

export const formPageWrapper: CSSProperties = {
  maxWidth: "400px",
  minHeight: "100vh",
  margin: "0 auto",
  padding: "28px",
  boxSizing: "border-box",
  display: "grid",
  alignContent: "center",
};

export const formCard: CSSProperties = {
  padding: "28px",
  border: "1px solid var(--color-border)",
  borderRadius: "18px",
  backgroundColor: "var(--color-surface)",
  boxShadow: "var(--shadow-card)",
};

export const headerRow: CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  marginBottom: "24px",
};

export const centeredFullScreen: CSSProperties = {
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  height: "100vh",
};

export const section: CSSProperties = {
  marginBottom: "32px",
};
