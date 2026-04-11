import { useCallback } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type AuthUser = components["schemas"]["MeSuccess"];

async function fetchMe(): Promise<AuthUser | null> {
  const { data, error, response } = await api.GET("/api/auth/me");
  if (response.status === 401) return null;
  if (error) throw new Error(error.message);
  return data ?? null;
}

export function useAuth() {
  const queryClient = useQueryClient();
  const { data: user, isLoading } = useQuery({
    queryKey: queryKeys.authMe,
    queryFn: fetchMe,
    staleTime: Infinity,
    retry: false,
  });

  const refresh = useCallback(async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.authMe });
  }, [queryClient]);

  const logout = useCallback(async () => {
    await api.POST("/api/auth/logout");
    queryClient.setQueryData(queryKeys.authMe, null);
  }, [queryClient]);

  return {
    isAuthenticated: user != null,
    username: user?.username ?? null,
    role: user?.role ?? null,
    mustChangePassword: user?.must_change_password ?? false,
    isLoading,
    logout,
    refresh,
  };
}
