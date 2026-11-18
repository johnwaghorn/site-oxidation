import { useRef } from "react";
import { CloseIcon, SearchIcon } from "../icons";

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
}

export function SearchInput({
  value,
  onChange,
  placeholder,
}: SearchInputProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  function clearSearchAndKeepFocusInTheBox() {
    onChange("");
    inputRef.current?.focus();
  }

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
        ref={inputRef}
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
          onClick={clearSearchAndKeepFocusInTheBox}
          className="form-field-button"
          style={{ minWidth: "32px", marginRight: "-8px", padding: 0 }}
        >
          <CloseIcon width="15" height="15" />
        </button>
      )}
    </div>
  );
}
