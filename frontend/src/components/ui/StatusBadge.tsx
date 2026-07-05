import type { components } from "../../generated/schema";

type SiteStatus = components["schemas"]["SiteStatus"];

interface StatusBadgeProps {
  status: SiteStatus;
}

const statusConfig = {
  pending: {
    label: "PENDING",
    title: "Not checked yet",
    backgroundColor: "var(--color-warning-bg)",
    color: "var(--color-warning-text)",
  },
  up: {
    label: "UP",
    title: "Responding as expected",
    backgroundColor: "var(--color-success-bg)",
    color: "var(--color-success-text)",
  },
  down: {
    label: "DOWN",
    title:
      "Not responding as expected (unreachable, wrong status code, or missing expected text)",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  blocked: {
    label: "BLOCKED",
    title:
      "Probe skipped by policy, usually because the host resolves to a private/internal IP",
    backgroundColor: "var(--color-neutral-bg)",
    color: "var(--color-neutral-text)",
  },
};

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = statusConfig[status];
  return (
    <span
      title={config.title}
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
