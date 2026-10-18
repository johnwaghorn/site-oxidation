import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type VersionResponse = components["schemas"]["VersionResponse"];

async function fetchVersion(): Promise<VersionResponse["version"] | null> {
  const { data, error } = await api.GET("/api/version");
  if (error) return null;
  return data.version;
}

export function useVersion() {
  const { data: version } = useQuery({
    queryKey: queryKeys.version,
    queryFn: fetchVersion,
    staleTime: Infinity,
    retry: false,
  });
  return version ?? null;
}
