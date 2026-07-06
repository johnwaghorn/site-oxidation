import type { ReactNode } from "react";
import { SearchIcon } from "../icons";

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}

interface SearchToolbarProps {
  children: ReactNode;
  action?: ReactNode;
}

export function SearchToolbar({ children, action }: SearchToolbarProps) {
  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        alignItems: "center",
        gap: "12px",
        marginBottom: "20px",
      }}
    >
      {children}
      {action && <div style={{ marginLeft: "auto" }}>{action}</div>}
    </div>
  );
}

export function SearchInput({
  value,
  onChange,
  placeholder,
}: SearchInputProps) {
  return (
    <div
      className="form-field"
      style={{
        display: "flex",
        alignItems: "center",
        flex: "1 1 320px",
        maxWidth: "480px",
        minWidth: 0,
        padding: "0 10px",
      }}
    >
      <SearchIcon style={{ flexShrink: 0, color: "var(--color-muted)" }} />
      <input
        type="text"
        aria-label={placeholder}
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="form-field-inner"
        style={{ padding: "10px 8px" }}
      />
      {value && (
        <button
          type="button"
          aria-label="Clear search"
          onClick={() => onChange("")}
          className="form-field-button"
          style={{ padding: "2px 4px", fontSize: "18px", lineHeight: 1 }}
        >
          &times;
        </button>
      )}
    </div>
  );
}
