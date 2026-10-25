import { Link } from "react-router-dom";

interface CanaryBadgeProps {
  isAdmin: boolean;
}

const EXPLANATION =
  "This site is also used to check Site Oxidation's own internet connection. If it goes down, the outage may not be recorded.";

const badgeStyle = {
  backgroundColor: "var(--color-warning-bg)",
  color: "var(--color-warning-text)",
};

export function CanaryBadge({ isAdmin }: CanaryBadgeProps) {
  if (isAdmin) {
    return (
      <Link
        to="/admin/canary"
        title={`${EXPLANATION} Point the canary at a site you don't monitor.`}
        className="badge"
        style={{ ...badgeStyle, textDecoration: "none" }}
      >
        CANARY
      </Link>
    );
  }
  return (
    <span
      title={`${EXPLANATION} Ask an admin to point the canary at a site you don't monitor.`}
      className="badge"
      style={badgeStyle}
    >
      CANARY
    </span>
  );
}
