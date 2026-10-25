import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type SiteImportRequest = components["schemas"]["SiteImportRequest"];

interface ImportSitesVariables {
  teamId: number | null;
  file: SiteImportRequest;
}

export function useExportSites() {
  return useMutation({
    mutationFn: async () => {
      const { data, error } = await api.GET("/api/sites/export");
      if (error) throw new Error(error.message);
      return data;
    },
  });
}

export function useImportSites() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ teamId, file }: ImportSitesVariables) => {
      const { data, error } = await api.POST("/api/sites/import", {
        params: { query: { team_id: teamId ?? undefined } },
        body: file,
      });
      if (error) throw new Error(error.message);
      return data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.sitesAll });
    },
  });
}
