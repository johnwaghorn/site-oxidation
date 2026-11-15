import {
  keepPreviousData,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";

export function useAdminNotificationOutbox(page: number, perPage: number) {
  return useQuery({
    queryKey: queryKeys.adminNotificationOutbox(page, perPage),
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/notification-outbox", {
        params: { query: { page, per_page: perPage } },
      });
      if (error) throw new Error(error.message);
      return data!;
    },
    placeholderData: keepPreviousData,
    refetchInterval: 30_000,
  });
}

export function useCancelNotificationOutboxEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: number) => {
      const { error } = await api.DELETE(
        "/api/admin/notification-outbox/{id}",
        {
          params: { path: { id } },
        },
      );
      if (error) throw new Error(error.message);
    },
    onSettled: async () => {
      await queryClient.invalidateQueries({
        queryKey: queryKeys.adminNotificationOutboxAll,
      });
    },
  });
}
