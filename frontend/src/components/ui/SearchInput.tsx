import { useState, type ReactNode } from "react";
import { polishedFieldChrome, polishedFieldFocus } from "../../lib/styles";

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
        justifyContent: "space-between",
        gap: "12px",
        marginBottom: "20px",
      }}
    >
      {children}
      {action}
    </div>
  );
}

export function SearchInput({
  value,
  onChange,
  placeholder,
}: SearchInputProps) {
  const [isFocused, setIsFocused] = useState(false);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        flex: "1 1 320px",
        maxWidth: "480px",
        minWidth: 0,
        ...polishedFieldChrome,
        ...(isFocused ? polishedFieldFocus : null),
        padding: "0 10px",
      }}
    >
      <svg
        aria-hidden="true"
        viewBox="0 0 24 24"
        width="17"
        height="17"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
        style={{ flexShrink: 0, color: "#9ca3af" }}
      >
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" />
      </svg>
      <input
        type="text"
        aria-label={placeholder}
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setIsFocused(false)}
        style={{
          width: "100%",
          minWidth: 0,
          padding: "10px 8px",
          border: "none",
          outline: "none",
          background: "transparent",
          color: "inherit",
          fontSize: "14px",
        }}
      />
      {value && (
        <button
          type="button"
          aria-label="Clear search"
          onClick={() => onChange("")}
          style={{
            padding: "2px 4px",
            border: "none",
            background: "transparent",
            color: "#9ca3af",
            cursor: "pointer",
            fontSize: "18px",
            lineHeight: 1,
          }}
        >
          &times;
        </button>
      )}
    </div>
  );
}
