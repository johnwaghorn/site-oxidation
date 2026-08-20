import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { components } from "../generated/schema";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";

type UpdateCanarySettingsRequest =
  components["schemas"]["UpdateCanarySettingsRequest"];

export function useAdminCanary() {
  return useQuery({
    queryKey: queryKeys.adminCanary,
    queryFn: async () => {
      const { data, error } = await api.GET("/api/admin/canary");
      if (error) throw new Error(error.message);
      return data!;
    },
    refetchInterval: 30_000,
  });
}

function useCanaryMutation<Variables>(
  mutationFn: (
    variables: Variables,
  ) => Promise<components["schemas"]["CanarySettingsResponse"]>,
) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onSuccess: (settings) => {
      queryClient.setQueryData(queryKeys.adminCanary, settings);
    },
  });
}

export function useUpdateAdminCanary() {
  return useCanaryMutation(async (settings: UpdateCanarySettingsRequest) => {
    const { data, error } = await api.PUT("/api/admin/canary", {
      body: settings,
    });
    if (error) throw new Error(error.message);
    return data!;
  });
}

export function useTestAdminCanary() {
  return useCanaryMutation<void>(async () => {
    const { data, error } = await api.POST("/api/admin/canary/test");
    if (error) throw new Error(error.message);
    return data!;
  });
}
