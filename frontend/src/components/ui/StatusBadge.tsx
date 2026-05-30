type SiteStatus = "pending" | "up" | "down" | "blocked";

interface StatusBadgeProps {
  status: SiteStatus;
}

const statusConfig = {
  pending: {
    label: "PENDING",
    title: "Not checked yet",
    backgroundColor: "#dbeafe",
    color: "#1e40af",
  },
  up: {
    label: "UP",
    title: "Responding as expected",
    backgroundColor: "#dcfce7",
    color: "#166534",
  },
  down: {
    label: "DOWN",
    title:
      "Not responding as expected (unreachable, wrong status code, or missing expected text)",
    backgroundColor: "#fee2e2",
    color: "#991b1b",
  },
  blocked: {
    label: "BLOCKED",
    title:
      "Probe skipped by policy, usually because the host resolves to a private/internal IP",
    backgroundColor: "#f3f4f6",
    color: "#6b7280",
  },
};

export function StatusBadge({ status }: StatusBadgeProps) {
  const config = statusConfig[status];
  return (
    <span
      title={config.title}
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
