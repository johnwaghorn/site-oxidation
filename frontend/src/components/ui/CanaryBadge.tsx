import type { components } from "../../generated/schema";
import { describeCanaryWarning } from "../../lib/canary";

type CanaryWarning = components["schemas"]["CanaryWarning"];

interface CanaryBadgeProps {
  warning: CanaryWarning;
  since?: string | null;
  isAdmin: boolean;
}

const warningConfig: Record<
  CanaryWarning,
  { label: string; backgroundColor: string; color: string }
> = {
  inactive: {
    label: "NO ACTIVE CANARY",
    backgroundColor: "var(--color-danger-bg)",
    color: "var(--color-danger-text)",
  },
  shares_host: {
    label: "CANARY",
    backgroundColor: "var(--color-warning-bg)",
    color: "var(--color-warning-text)",
  },
};

export function CanaryBadge({ warning, since, isAdmin }: CanaryBadgeProps) {
  const config = warningConfig[warning];
  const { explanation, action } = describeCanaryWarning(
    warning,
    since,
    isAdmin,
  );
  return (
    <span
      title={`${explanation} ${action}`}
      className="badge"
      style={{ backgroundColor: config.backgroundColor, color: config.color }}
    >
      {config.label}
    </span>
  );
}
