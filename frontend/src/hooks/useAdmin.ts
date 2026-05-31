import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components, operations } from "../generated/schema";

type ListUsersQuery = NonNullable<
  operations["list_users"]["parameters"]["query"]
>;

export interface AdminUsersFilters {
  page: NonNullable<ListUsersQuery["page"]>;
  perPage: NonNullable<ListUsersQuery["per_page"]>;
  search?: ListUsersQuery["search"];
  teamId?: ListUsersQuery["team_id"];
  excludeTeamId?: ListUsersQuery["exclude_team_id"];
  active?: ListUsersQuery["active"];
}

type CreateTeamRequest = components["schemas"]["CreateTeamRequest"];
type UpdateTeamRequest = components["schemas"]["UpdateTeamRequest"];
type AddMemberRequest = components["schemas"]["AddMemberRequest"];
type CreateUserRequest = components["schemas"]["CreateUserRequest"];
type UpdateUserRequest = components["schemas"]["UpdateUserRequest"];
type ResetPasswordRequest = components["schemas"]["ResetPasswordRequest"];

export function useAdminTeams(page = 1, perPage = 20) {
  return useQuery({
    queryKey: queryKeys.adminTeams(page, perPage),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/teams", {
        params: { query: { page, per_page: perPage } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    placeholderData: keepPreviousData,
  });
}

export function useAdminTeam(id: number) {
  return useQuery({
    queryKey: queryKeys.adminTeam(id),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/teams/{id}", {
        params: { path: { id } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    enabled: id > 0,
  });
}

export function useAdminTeamSites(id: number, page = 1, perPage = 20) {
  return useQuery({
    queryKey: queryKeys.adminTeamSites(id, page, perPage),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/teams/{id}/sites", {
        params: { path: { id }, query: { page, per_page: perPage } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    enabled: id > 0,
    placeholderData: keepPreviousData,
  });
}

export function useTeamOptions(search: string) {
  return useQuery({
    queryKey: queryKeys.adminTeamOptions(search),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/teams/options", {
        params: { query: { search: search || undefined } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    placeholderData: keepPreviousData,
  });
}

export function useAdminUsers(filters: AdminUsersFilters) {
  return useQuery({
    queryKey: queryKeys.adminUsers(filters),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/users", {
        params: {
          query: {
            page: filters.page,
            per_page: filters.perPage,
            search: filters.search || undefined,
            team_id: filters.teamId,
            exclude_team_id: filters.excludeTeamId,
            active: filters.active,
          },
        },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    placeholderData: keepPreviousData,
  });
}

export function useCreateTeam() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (team: CreateTeamRequest) => {
      const { data, error } = await api.POST("/api/admin/teams", {
        body: team,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.authMe });
    },
  });
}

export function useUpdateTeam() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      team,
    }: {
      id: number;
      team: UpdateTeamRequest;
    }) => {
      const { data, error } = await api.PATCH("/api/admin/teams/{id}", {
        params: { path: { id } },
        body: team,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.authMe });
    },
  });
}

export function useDeleteTeam() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: number) => {
      const { error } = await api.DELETE("/api/admin/teams/{id}", {
        params: { path: { id } },
      });
      if (error) throw new Error(error.message);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.authMe });
    },
  });
}

export function useUnassignTeamSite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      teamId,
      siteId,
    }: {
      teamId: number;
      siteId: number;
    }) => {
      const { error } = await api.DELETE(
        "/api/admin/teams/{id}/sites/{site_id}",
        {
          params: { path: { id: teamId, site_id: siteId } },
        },
      );
      if (error) throw new Error(error.message);
    },
    onSuccess: async (_, { teamId, siteId }) => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeam(teamId),
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamSitesAll(teamId),
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({ queryKey: queryKeys.sitesAll });
      await queryClient.invalidateQueries({ queryKey: queryKeys.site(siteId) });
    },
  });
}

export function useAddTeamMember() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      teamId,
      member,
    }: {
      teamId: number;
      member: AddMemberRequest;
    }) => {
      const { data, error } = await api.POST("/api/admin/teams/{id}/members", {
        params: { path: { id: teamId } },
        body: member,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
    },
  });
}

export function useRemoveTeamMember() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      teamId,
      userId,
    }: {
      teamId: number;
      userId: number;
    }) => {
      const { error } = await api.DELETE(
        "/api/admin/teams/{id}/members/{user_id}",
        {
          params: { path: { id: teamId, user_id: userId } },
        },
      );
      if (error) throw new Error(error.message);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
    },
  });
}

export function useCreateUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (user: CreateUserRequest) => {
      const { data, error } = await api.POST("/api/admin/users", {
        body: user,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
    },
  });
}

export function useUpdateUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      user,
    }: {
      id: number;
      user: UpdateUserRequest;
    }) => {
      const { data, error } = await api.PATCH("/api/admin/users/{id}", {
        params: { path: { id } },
        body: user,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
    },
  });
}

export function useDeleteUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: number) => {
      const { error } = await api.DELETE("/api/admin/users/{id}", {
        params: { path: { id } },
      });
      if (error) throw new Error(error.message);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
    },
  });
}

export function useResetPassword() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      id,
      payload,
    }: {
      id: number;
      payload: ResetPasswordRequest;
    }) => {
      const { data, error } = await api.POST(
        "/api/admin/users/{id}/reset-password",
        {
          params: { path: { id } },
          body: payload,
        },
      );
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminUsersAll,
      });
    },
  });
}
