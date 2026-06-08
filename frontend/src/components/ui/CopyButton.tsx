import { useState } from "react";

interface CopyButtonProps {
  value: string;
}

async function copyText(value: string): Promise<void> {
  if (navigator.clipboard) {
    try {
      await navigator.clipboard.writeText(value);
      return;
    } catch {
      // Clipboard access can still be rejected by browser permissions
    }
  }

  const textarea = document.createElement("textarea");
  textarea.value = value;
  textarea.style.position = "fixed";
  textarea.style.opacity = "0";
  document.body.appendChild(textarea);
  textarea.select();
  const copied = document.execCommand("copy");
  document.body.removeChild(textarea);
  if (!copied) {
    throw new Error("Copy failed");
  }
}

export function CopyButton({ value }: CopyButtonProps) {
  const [status, setStatus] = useState<"idle" | "copied" | "failed">("idle");

  const handleCopy = async () => {
    try {
      await copyText(value);
      setStatus("copied");
    } catch {
      setStatus("failed");
    }
    window.setTimeout(() => setStatus("idle"), 2000);
  };

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
      {status !== "idle" && (
        <span
          role="status"
          style={{
            padding: "4px 8px",
            borderRadius: "5px",
            backgroundColor:
              status === "copied"
                ? "var(--color-success-bg)"
                : "var(--color-danger-bg)",
            color:
              status === "copied"
                ? "var(--color-success-text)"
                : "var(--color-danger-text)",
            fontSize: "12px",
            fontWeight: 600,
          }}
        >
          {status === "copied" ? "Copied!" : "Copy failed"}
        </span>
      )}
      <button
        type="button"
        aria-label={status === "copied" ? "Copied" : "Copy to clipboard"}
        title={status === "failed" ? "Copy failed. Try again." : "Copy"}
        onClick={handleCopy}
        style={{
          display: "grid",
          placeItems: "center",
          width: "38px",
          height: "38px",
          padding: 0,
          border: `1px solid ${
            status === "copied" ? "var(--color-success)" : "var(--color-border)"
          }`,
          borderRadius: "7px",
          backgroundColor: "var(--color-surface-elevated)",
          color:
            status === "copied" ? "var(--color-success)" : "var(--color-muted)",
          cursor: "pointer",
        }}
      >
        {status === "copied" ? <CheckIcon /> : <CopyIcon />}
      </button>
    </div>
  );
}

function CopyIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <rect width="14" height="14" x="8" y="8" rx="2" />
      <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="m5 12 4 4L19 6" />
    </svg>
  );
}
