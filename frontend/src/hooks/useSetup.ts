import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type BootstrapResponse = components["schemas"]["BootstrapResponse"];

async function fetchSetupStatus(): Promise<boolean> {
  try {
    const { data, response } = await api.GET("/api/setup/status");
    if (!response.ok) return false;
    return data?.setup_required ?? false;
  } catch {
    return false;
  }
}

export function useSetupStatus() {
  const queryClient = useQueryClient();
  const { data: setupRequired, isLoading } = useQuery({
    queryKey: queryKeys.setupStatus,
    queryFn: fetchSetupStatus,
    staleTime: Infinity,
    retry: false,
  });

  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.setupStatus });
  }, [queryClient]);

  return {
    setupRequired: setupRequired ?? null,
    isLoading,
    refresh,
  };
}

export function useBootstrap() {
  return useMutation({
    mutationFn: async (): Promise<BootstrapResponse> => {
      const { data, error } = await api.POST("/api/setup/bootstrap");
      if (error) throw error;
      return data!;
    },
  });
}
