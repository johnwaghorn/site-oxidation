import type { CSSProperties } from "react";

export const table: CSSProperties = {
  width: "100%",
  borderCollapse: "separate",
  borderSpacing: 0,
  overflow: "hidden",
  border: "1px solid var(--color-border)",
  borderRadius: "14px",
  backgroundColor: "var(--color-surface)",
  boxShadow: "var(--shadow-card)",
};

export const tableHeaderRow: CSSProperties = {
  boxShadow: "inset 0 -1px 0 var(--color-border-strong)",
};

export const tableRow: CSSProperties = {
  boxShadow: "inset 0 -1px 0 var(--color-border)",
};

export const tableCell: CSSProperties = {
  padding: "12px 14px",
};

export const tableCellLeft: CSSProperties = {
  padding: "12px 14px",
  textAlign: "left",
};

export const tableCellCenter: CSSProperties = {
  padding: "12px 14px",
  textAlign: "center",
};

export const tableCellRight: CSSProperties = {
  padding: "12px 14px",
  textAlign: "right",
};
