import type { components } from "../generated/schema";

type SiteStatus = components["schemas"]["SiteStatus"];

const STATUS_DESCRIPTIONS: Record<SiteStatus, string> = {
  pending: "Not checked yet",
  up: "Responding as expected",
  down: "Not responding as expected (unreachable, wrong status code, or missing expected text)",
  blocked:
    "Probe skipped by policy, usually because the host resolves to a private/internal IP",
};

export function describeSiteStatus(status: SiteStatus) {
  return STATUS_DESCRIPTIONS[status];
}
