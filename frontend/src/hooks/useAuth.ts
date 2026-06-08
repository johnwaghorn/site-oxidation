import { useCallback } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import { queryKeys } from "../lib/queryKeys";
import type { components } from "../generated/schema";

type AuthUser = components["schemas"]["MeSuccess"];
type ThemePreference = components["schemas"]["ThemePreference"];

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

  const updateThemePreference = useCallback(
    async (themePreference: ThemePreference) => {
      const previous = queryClient.getQueryData<AuthUser | null>(
        queryKeys.authMe,
      );

      queryClient.setQueryData<AuthUser | null>(queryKeys.authMe, (current) =>
        current ? { ...current, theme_preference: themePreference } : current,
      );

      const { error } = await api.PATCH("/api/auth/theme", {
        body: { theme_preference: themePreference },
      });
      if (error) {
        queryClient.setQueryData(queryKeys.authMe, previous);
      }
    },
    [queryClient],
  );

  return {
    isAuthenticated: user != null,
    username: user?.username ?? null,
    role: user?.role ?? null,
    mustChangePassword: user?.must_change_password ?? false,
    themePreference: user?.theme_preference ?? null,
    teams: user?.teams ?? [],
    isLoading,
    logout,
    refresh,
    updateThemePreference,
  };
}
