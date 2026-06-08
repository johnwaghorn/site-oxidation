import type { components } from "../../generated/schema";

type CertStatus = components["schemas"]["CertStatus"];

interface CertBadgeProps {
  status?: CertStatus | null;
  expiresAt?: string | null;
}

const statusConfig: Record<
  CertStatus,
  { label: string; backgroundColor: string; color: string }
> = {
  valid: {
    label: "VALID",
    backgroundColor: "var(--color-success-bg)",
    color: "var(--color-success-text)",
  },
  expiring: {
    label: "EXPIRING",
    backgroundColor: "var(--color-warning-bg)",
    color: "var(--color-warning-text)",
  },
  critical: {
    label: "CRITICAL",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  expired: {
    label: "EXPIRED",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  invalid: {
    label: "INVALID",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  none: {
    label: "NO TLS",
    backgroundColor: "var(--color-neutral-bg)",
    color: "var(--color-neutral-text)",
  },
};

const unknownConfig = {
  label: "-",
  backgroundColor: "var(--color-neutral-bg)",
  color: "var(--color-neutral-text)",
};

function tooltip(
  status: CertStatus | null | undefined,
  expiresAt?: string | null,
): string {
  if (status == null)
    return "Certificate status unavailable - not checked yet, blocked, or the certificate could not be read";
  switch (status) {
    case "none":
      return "Plain HTTP - no certificate";
    case "invalid":
      return "Invalid certificate: untrusted, hostname mismatch, or unreachable";
    case "expired":
      return expiresAt
        ? `Expired on ${new Date(expiresAt).toLocaleDateString()}`
        : "Certificate expired";
    default: {
      if (!expiresAt) return "Certificate valid";
      const expiry = new Date(expiresAt);
      const days = Math.ceil((expiry.getTime() - Date.now()) / 86400000);
      return `Valid until ${expiry.toLocaleDateString()} (${days} day${
        days === 1 ? "" : "s"
      })`;
    }
  }
}

export function CertBadge({ status, expiresAt }: CertBadgeProps) {
  const config = status ? statusConfig[status] : unknownConfig;
  return (
    <span
      title={tooltip(status, expiresAt)}
      style={{
        display: "inline-block",
        padding: "2px 8px",
        borderRadius: "4px",
        fontSize: "12px",
        fontWeight: 500,
        backgroundColor: config.backgroundColor,
        color: config.color,
      }}
    >
      {config.label}
    </span>
  );
}
