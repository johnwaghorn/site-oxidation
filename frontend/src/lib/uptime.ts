const FULL_UPTIME_BASIS_POINTS = 10_000;
const MILLISECONDS_IN_A_DAY = 24 * 60 * 60 * 1000;

export function formatUptime(basisPoints: number | null | undefined) {
  if (basisPoints == null) return "-";
  if (basisPoints === FULL_UPTIME_BASIS_POINTS) return "100%";
  return `${(basisPoints / 100).toFixed(2)}%`;
}

export function describeUptimeCoverage(createdAt: string, windowDays: number) {
  const added = new Date(createdAt);
  const windowStart = Date.now() - windowDays * MILLISECONDS_IN_A_DAY;
  return added.getTime() > windowStart
    ? `Uptime since the site was added on ${added.toLocaleDateString()}`
    : `Uptime over the last ${windowDays} days`;
}

export function formatDuration(totalSeconds: number) {
  if (totalSeconds < 60) return `${totalSeconds}s`;
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours >= 48) return `${Math.floor(hours / 24)}d ${hours % 24}h`;
  if (hours === 0) return `${minutes}m`;
  return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
}
