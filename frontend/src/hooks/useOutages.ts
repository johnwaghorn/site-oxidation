import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";

export function useOutages(siteId: number, page = 1, perPage = 20) {
  return useQuery({
    queryKey: ["outages", siteId, { page, perPage }],
    queryFn: async () => {
      const { data, error } = await api.GET("/api/sites/{id}/outages", {
        params: {
          path: { id: siteId },
          query: { page, per_page: perPage },
        },
      });
      if (error) throw error;
      return data!;
    },
    enabled: siteId > 0,
  });
}
