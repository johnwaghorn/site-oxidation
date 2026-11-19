import type { components } from "../generated/schema";

type CanaryWarning = components["schemas"]["CanaryWarning"];

const WARNING_TEXT: Record<
  CanaryWarning,
  { explanation: string; adminAction: string; userAction: string }
> = {
  inactive: {
    explanation:
      "No connectivity canary is active. If the monitor loses internet access, sites may be incorrectly recorded as down.",
    adminAction: "Configure the canary.",
    userAction: "Ask an admin to configure the canary.",
  },
  shares_host: {
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

export function describeCanaryWarning(
  warning: CanaryWarning,
  since: string | null | undefined,
  isAdmin: boolean,
) {
  const text = WARNING_TEXT[warning];
  return {
    explanation: [text.explanation, describeSince(warning, since)]
      .filter(Boolean)
      .join(" "),
    action: isAdmin ? text.adminAction : text.userAction,
  };
}
