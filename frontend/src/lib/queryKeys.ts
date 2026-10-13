import type { AdminUsersFilters } from "../hooks/useAdmin";
import type { SitesFilters } from "../hooks/useSites";

export const queryKeys = {
  authMe: ["auth", "me"] as const,
  sites: (filters: SitesFilters) => ["sites", filters] as const,
  sitesUnassignedCount: ["sites", "unassigned-count"] as const,
  sitesAll: ["sites"] as const,
  site: (id: number) => ["sites", id] as const,
  outages: (siteId: number, page: number, perPage: number) =>
    ["sites", siteId, "outages", { page, perPage }] as const,
  teamNotifications: (teamId: number) =>
    ["teams", teamId, "notifications"] as const,
  setupStatus: ["setup", "status"] as const,
  adminTeamsAll: ["admin", "teams"] as const,
  adminTeams: (page: number, perPage: number, search?: string) =>
    ["admin", "teams", { page, perPage, search }] as const,
  adminTeam: (id: number) => ["admin", "teams", id] as const,
  adminTeamSitesAll: (id: number) => ["admin", "teams", id, "sites"] as const,
  adminTeamSites: (
    id: number,
    page: number,
    perPage: number,
    search?: string,
  ) => ["admin", "teams", id, "sites", { page, perPage, search }] as const,
  adminTeamOptions: (search: string) =>
    ["admin", "teams", "options", search] as const,
  adminUsersAll: ["admin", "users"] as const,
  adminUsers: (filters: AdminUsersFilters) =>
    ["admin", "users", filters] as const,
  adminCanary: ["admin", "canary"] as const,
};
