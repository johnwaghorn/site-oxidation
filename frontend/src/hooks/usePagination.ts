import { useCallback, useEffect, useRef } from "react";
import { useSearchParams } from "react-router-dom";

const PAGE_PARAM = "page";
const PER_PAGE = 20;

function pageFromUrl(searchParams: URLSearchParams) {
  const page = Number(searchParams.get(PAGE_PARAM));
  return Number.isInteger(page) && page >= 1 ? page : 1;
}

export function usePagination(resetKey = "") {
  const [searchParams, setSearchParams] = useSearchParams();
  const page = pageFromUrl(searchParams);

  const setPage = useCallback(
    (nextPage: number, replace: boolean) => {
      setSearchParams(
        (current) => {
          const next = new URLSearchParams(current);
          if (nextPage > 1) next.set(PAGE_PARAM, String(nextPage));
          else next.delete(PAGE_PARAM);
          return next;
        },
        { replace },
      );
    },
    [setSearchParams],
  );

  const goToPage = useCallback(
    (nextPage: number) => setPage(nextPage, false),
    [setPage],
  );

  const correctPage = useCallback(
    (nextPage: number) => setPage(nextPage, true),
    [setPage],
  );

  const previousResetKey = useRef(resetKey);
  useEffect(() => {
    if (previousResetKey.current === resetKey) return;
    previousResetKey.current = resetKey;
    correctPage(1);
  }, [resetKey, correctPage]);

  return { page, perPage: PER_PAGE, goToPage, correctPage };
}
