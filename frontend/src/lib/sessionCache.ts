import { hashKey, type QueryClient } from "@tanstack/react-query";
import { queryKeys } from "./queryKeys";

export function forgetEverythingLoadedForTheSignedInUser(
  queryClient: QueryClient,
) {
  const whoIsSignedInQueryHash = hashKey(queryKeys.authMe);
  queryClient.removeQueries({
    predicate: (query) => query.queryHash !== whoIsSignedInQueryHash,
  });
}
