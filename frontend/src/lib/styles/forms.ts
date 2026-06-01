import type { CSSProperties } from "react";

export const formColumn: CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "16px",
};

export const formInput: CSSProperties = {
  padding: "12px",
  fontSize: "16px",
};

export const inlineForm: CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  gap: "8px",
  marginBottom: "24px",
};

export const compactInput: CSSProperties = {
  padding: "8px",
};

export const polishedFieldChrome: CSSProperties = {
  boxSizing: "border-box",
  padding: "10px 12px",
  border: "1px solid GrayText",
  borderRadius: "8px",
  backgroundColor: "Field",
  color: "FieldText",
  fontSize: "14px",
  transition: "border-color 150ms ease, box-shadow 150ms ease",
};

export const polishedFieldFocus: CSSProperties = {
  borderColor: "#646cff",
  boxShadow: "0 0 0 3px rgba(100, 108, 255, 0.2)",
  outline: "none",
};

export const comboboxList: CSSProperties = {
  position: "absolute",
  zIndex: 10,
  top: "calc(100% + 2px)",
  left: 0,
  right: 0,
  margin: 0,
  padding: 0,
  listStyle: "none",
  maxHeight: "200px",
  overflowY: "auto",
  border: "1px solid GrayText",
  borderRadius: "4px",
  backgroundColor: "Canvas",
};

export const comboboxItem: CSSProperties = {
  display: "block",
  width: "100%",
  padding: "8px 10px",
  border: "none",
  borderRadius: 0,
  background: "none",
  textAlign: "left",
  fontSize: "14px",
  fontWeight: 400,
  color: "inherit",
  cursor: "pointer",
};

export const comboboxItemHovered: CSSProperties = {
  backgroundColor: "Highlight",
  color: "HighlightText",
};
