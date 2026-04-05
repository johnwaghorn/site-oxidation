export const queryKeys = {
  authMe: ["auth", "me"] as const,
  sites: (page: number, perPage: number) =>
    ["sites", { page, perPage }] as const,
  sitesAll: ["sites"] as const,
  site: (id: number) => ["sites", id] as const,
  outages: (siteId: number, page: number, perPage: number) =>
    ["sites", siteId, "outages", { page, perPage }] as const,
};
