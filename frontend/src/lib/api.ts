import type { QueryClient } from "@tanstack/react-query";
import createClient from "openapi-fetch";
import type { paths } from "../generated/schema";

const AUTH_PATH_PREFIX = "/api/auth/";

let queryClient: QueryClient | null = null;

export function setQueryClient(client: QueryClient) {
  queryClient = client;
}

function getPathname(input: RequestInfo | URL): string {
  if (typeof input === "string") {
    return new URL(input, window.location.origin).pathname;
  }
  if (input instanceof URL) {
    return input.pathname;
  }
  return new URL(input.url, window.location.origin).pathname;
}

const fetchWithCredentials: typeof fetch = async (input, init) => {
  const response = await fetch(input, { ...init, credentials: "include" });
  if (response.status === 401 && queryClient) {
    const pathname = getPathname(input);
    if (!pathname.startsWith(AUTH_PATH_PREFIX)) {
      queryClient.setQueryData(["auth", "me"], null);
    }
  }
  return response;
};

export const api = createClient<paths>({
  baseUrl: "",
  fetch: fetchWithCredentials,
});
