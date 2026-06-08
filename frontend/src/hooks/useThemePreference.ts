import { useEffect, useState } from "react";

export type ThemePreference = "system" | "light" | "dark";

const storageKey = "site-oxidation-theme";

function readThemePreference(): ThemePreference {
  const stored = window.localStorage.getItem(storageKey);
  return stored === "light" || stored === "dark" || stored === "system"
    ? stored
    : "system";
}

export function useThemePreference() {
  const [themePreference, setThemePreference] =
    useState<ThemePreference>(readThemePreference);

  useEffect(() => {
    if (themePreference === "system") {
      window.localStorage.removeItem(storageKey);
      document.documentElement.removeAttribute("data-theme");
      return;
    }

    window.localStorage.setItem(storageKey, themePreference);
    document.documentElement.dataset.theme = themePreference;
  }, [themePreference]);

  return { themePreference, setThemePreference };
}
