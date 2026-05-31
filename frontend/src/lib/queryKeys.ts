import type { AdminUsersFilters } from "../hooks/useAdmin";

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
  adminTeam: (id: number) => ["admin", "teams", id] as const,
  adminTeamSitesAll: (id: number) => ["admin", "teams", id, "sites"] as const,
  adminTeamSites: (id: number, page: number, perPage: number) =>
    ["admin", "teams", id, "sites", { page, perPage }] as const,
  adminTeamOptions: (search: string) =>
    ["admin", "teams", "options", search] as const,
  adminUsersAll: ["admin", "users"] as const,
  adminUsers: (filters: AdminUsersFilters) =>
    ["admin", "users", filters] as const,
};
