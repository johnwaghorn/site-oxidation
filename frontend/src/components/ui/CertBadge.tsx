import type { components } from "../../generated/schema";
import { describeCertificate } from "../../lib/certificate";

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

export function CertBadge({ status, expiresAt }: CertBadgeProps) {
  const config = status ? statusConfig[status] : unknownConfig;
  return (
    <span
      title={describeCertificate(status, expiresAt)}
      className="badge"
      style={{
        backgroundColor: config.backgroundColor,
        color: config.color,
      }}
    >
      {config.label}
    </span>
  );
}
