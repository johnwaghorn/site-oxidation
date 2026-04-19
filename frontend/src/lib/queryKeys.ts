export interface AdminUsersFilters {
  page: number;
  perPage: number;
  search?: string;
  teamId?: number;
}

export const queryKeys = {
  authMe: ["auth", "me"] as const,
  sites: (page: number, perPage: number) =>
    ["sites", { page, perPage }] as const,
  sitesAll: ["sites"] as const,
  site: (id: number) => ["sites", id] as const,
  outages: (siteId: number, page: number, perPage: number) =>
    ["sites", siteId, "outages", { page, perPage }] as const,
  setupStatus: ["setup", "status"] as const,
  adminTeamsAll: ["admin", "teams"] as const,
  adminTeams: (page: number, perPage: number) =>
    ["admin", "teams", { page, perPage }] as const,
  adminUsersAll: ["admin", "users"] as const,
  adminUsers: (filters: AdminUsersFilters) =>
    ["admin", "users", filters] as const,
};
