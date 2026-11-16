import { useEffect } from "react";

const APP_NAME = "Site Oxidation";

export function usePageTitle(pageName: string | undefined) {
  useEffect(() => {
    document.title = pageName ? `${pageName} - ${APP_NAME}` : APP_NAME;
  }, [pageName]);
}
