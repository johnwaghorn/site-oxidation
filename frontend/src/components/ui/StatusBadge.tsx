import type { components } from "../../generated/schema";
import { describeSiteStatus } from "../../lib/siteStatus";

type SiteStatus = components["schemas"]["SiteStatus"];

interface StatusBadgeProps {
  status: SiteStatus;
}

const statusConfig = {
  pending: {
    label: "PENDING",
    backgroundColor: "var(--color-warning-bg)",
    color: "var(--color-warning-text)",
  },
  up: {
    label: "UP",
    backgroundColor: "var(--color-success-bg)",
    color: "var(--color-success-text)",
  },
  down: {
    label: "DOWN",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  blocked: {
    label: "BLOCKED",
    backgroundColor: "var(--color-blocked-bg)",
    color: "var(--color-blocked-text)",
  },
};

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = statusConfig[status];
  return (
    <span
      title={describeSiteStatus(status)}
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
