import {
  useQuery,
  useMutation,
  useQueryClient,
  keepPreviousData,
} from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys, type AdminUsersFilters } from "../lib/queryKeys";
import type { components } from "../generated/schema";

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
