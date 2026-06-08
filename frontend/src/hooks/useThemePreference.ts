import { useEffect, useState } from "react";
import type { components } from "../generated/schema";

export type ThemePreference = components["schemas"]["ThemePreference"];
const prePaintThemeStorageKey = "site-oxidation-theme";

function readThemePreference(): ThemePreference {
  const stored = window.localStorage.getItem(prePaintThemeStorageKey);
  return stored === "light" || stored === "dark" || stored === "system"
    ? stored
    : "system";
}

export function useThemePreference() {
  const [themePreference, setThemePreference] =
    useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    if (themePreference === "system") {
      window.localStorage.removeItem(prePaintThemeStorageKey);
      document.documentElement.removeAttribute("data-theme");
      return;
    }

    window.localStorage.setItem(prePaintThemeStorageKey, themePreference);
    document.documentElement.dataset.theme = themePreference;
  }, [themePreference]);

  return { themePreference, setThemePreference };
}
