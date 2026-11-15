import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components, operations } from "../generated/schema";

type UpdateTeamNotificationsRequest =
  components["schemas"]["UpdateTeamNotificationsRequest"];
type ListTeamNotificationSummariesQuery = NonNullable<
  operations["list_team_notification_summaries"]["parameters"]["query"]
>;

export interface TeamNotificationSummariesFilters {
  page: NonNullable<ListTeamNotificationSummariesQuery["page"]>;
  perPage: NonNullable<ListTeamNotificationSummariesQuery["per_page"]>;
  search?: ListTeamNotificationSummariesQuery["search"];
  hasAlertChannel?: ListTeamNotificationSummariesQuery["has_alert_channel"];
}

export function useTeamNotificationSummaries(
  filters: TeamNotificationSummariesFilters,
) {
  return useQuery({
    queryKey: queryKeys.teamNotificationSummaries(filters),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/teams/notifications", {
        params: {
          query: {
            page: filters.page,
            per_page: filters.perPage,
            search: filters.search || undefined,
            has_alert_channel: filters.hasAlertChannel,
          },
        },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    placeholderData: keepPreviousData,
  });
}

export function useTeamNotifications(teamId: number) {
  return useQuery({
    queryKey: queryKeys.teamNotifications(teamId),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/teams/{id}/notifications", {
        params: { path: { id: teamId } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    enabled: teamId > 0,
  });
}

export function useTestEmailNotification() {
  return useMutation({
    mutationFn: async (teamId: number) => {
      const { data, error } = await api.POST(
        "/api/teams/{id}/notifications/test/email",
        { params: { path: { id: teamId } } },
      );
      if (error) throw new Error(error.message);
      return data!;
    },
  });
}

export function useTestSlackNotification() {
  return useMutation({
    mutationFn: async (teamId: number) => {
      const { data, error } = await api.POST(
        "/api/teams/{id}/notifications/test/slack",
        { params: { path: { id: teamId } } },
      );
      if (error) throw new Error(error.message);
      return data!;
    },
  });
}

export function useTestTeamsNotification() {
  return useMutation({
    mutationFn: async (teamId: number) => {
      const { data, error } = await api.POST(
        "/api/teams/{id}/notifications/test/teams",
        { params: { path: { id: teamId } } },
      );
      if (error) throw new Error(error.message);
      return data!;
    },
  });
}

export function useUpdateTeamNotifications() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      teamId,
      payload,
    }: {
      teamId: number;
      payload: UpdateTeamNotificationsRequest;
    }) => {
      const { data, error } = await api.PATCH("/api/teams/{id}/notifications", {
        params: { path: { id: teamId } },
        body: payload,
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    onSuccess: async (_, { teamId }) => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.teamNotifications(teamId),
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.teamNotificationSummariesAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeamsAll,
      });
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminTeam(teamId),
      });
    },
  });
}
