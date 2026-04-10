import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type CreateTeamRequest = components["schemas"]["CreateTeamRequest"];
type UpdateTeamRequest = components["schemas"]["UpdateTeamRequest"];
type AddMemberRequest = components["schemas"]["AddMemberRequest"];

export function useAdminTeams() {
  return useQuery({
    queryKey: queryKeys.adminTeams,
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/teams");
      if (error) throw error;
      return data!;
    },
  });
}

export function useAdminUsers() {
  return useQuery({
    queryKey: queryKeys.adminUsers,
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/users");
      if (error) throw error;
      return data!;
    },
  });
}

export function useCreateTeam() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (team: CreateTeamRequest) => {
      const { data, error } = await api.POST("/api/admin/teams", {
        body: team,
      });
      if (error) throw error;
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminTeams });
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
      if (error) throw error;
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminTeams });
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
      if (error) throw error;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminTeams });
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
      if (error) throw error;
      return data!;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminTeams });
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminUsers });
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
      if (error) throw error;
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminTeams });
      await queryClient.invalidateQueries({ queryKey: queryKeys.adminUsers });
    },
  });
}
