import { useEffect, useState } from "react";
import type { components } from "../generated/schema";

export type ThemePreference = components["schemas"]["ThemePreference"];
const prePaintThemeStorageKey = "site-oxidation-theme";

function readThemePreference(): ThemePreference {
  const stored = window.localStorage.getItem(prePaintThemeStorageKey);
  return stored === "light" || stored === "dark" || stored === "system"
    ? stored
    : "dark";
}

export function useThemePreference() {
  const [themePreference, setThemePreference] =
    useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    if (themePreference === "system") {
      window.localStorage.setItem(prePaintThemeStorageKey, themePreference);
      const media = window.matchMedia("(prefers-color-scheme: dark)");
      const applySystemTheme = () => {
        document.documentElement.dataset.theme = media.matches
          ? "dark"
          : "light";
      };
      applySystemTheme();
      media.addEventListener("change", applySystemTheme);
      return () => media.removeEventListener("change", applySystemTheme);
    }

    window.localStorage.setItem(prePaintThemeStorageKey, themePreference);
    document.documentElement.dataset.theme = themePreference;
  }, [themePreference]);

  return { themePreference, setThemePreference };
}
