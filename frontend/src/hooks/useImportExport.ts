import { useMutation } from "@tanstack/react-query";
import { api } from "../lib/api";

export function useExportSites() {
  return useMutation({
    mutationFn: async () => {
      const { data, error } = await api.GET("/api/sites/export");
      if (error) throw new Error(error.message);
      return data;
    },
  });
}
