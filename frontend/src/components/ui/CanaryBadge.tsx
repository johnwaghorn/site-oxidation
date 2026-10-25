import { Link } from "react-router-dom";
import type { components } from "../../generated/schema";

type CanaryWarning = components["schemas"]["CanaryWarning"];

interface CanaryBadgeProps {
  warning: CanaryWarning;
  since?: string | null;
  isAdmin: boolean;
}

const warningConfig: Record<
  CanaryWarning,
  {
    label: string;
    backgroundColor: string;
    color: string;
    explanation: string;
    adminAction: string;
    userAction: string;
  }
> = {
  inactive: {
    label: "NO ACTIVE CANARY",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
    explanation:
      "No connectivity canary is active. If the monitor loses internet access, sites may be incorrectly recorded as down.",
    adminAction: "Configure the canary.",
    userAction: "Ask an admin to configure the canary.",
  },
  shares_host: {
    label: "CANARY",
    backgroundColor: "var(--color-warning-bg)",
    color: "var(--color-warning-text)",
    explanation:
      "This site is also used to check Site Oxidation's own internet connection. If it goes down, the outage may not be recorded.",
    adminAction: "Point the canary at a site you don't monitor.",
    userAction: "Ask an admin to point the canary at a site you don't monitor.",
  },
};

function describeSince(warning: CanaryWarning, since?: string | null) {
  if (!since) {
    return warning === "inactive" ? "A canary has never been active." : "";
  }
  const date = new Date(since).toLocaleString();
  return warning === "inactive"
    ? `No canary has been active since ${date}.`
    : `The canary has checked this host since ${date}.`;
}

export function CanaryBadge({ warning, since, isAdmin }: CanaryBadgeProps) {
  const config = warningConfig[warning];
  const action = isAdmin ? config.adminAction : config.userAction;
  const title = [config.explanation, describeSince(warning, since), action]
    .filter(Boolean)
    .join(" ");
  const style = {
    backgroundColor: config.backgroundColor,
    color: config.color,
  };
  if (isAdmin) {
    return (
      <Link
        to="/admin/canary"
        title={title}
        className="badge"
        style={{ ...style, textDecoration: "none" }}
      >
        {config.label}
      </Link>
    );
  }
  return (
    <span title={title} className="badge" style={style}>
      {config.label}
    </span>
  );
}
