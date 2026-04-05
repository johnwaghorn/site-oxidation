import { useState, useCallback } from "react";

export function usePagination(initialPage = 1, initialPerPage = 20) {
  const [page, setPage] = useState(initialPage);
  const [perPage, setPerPage] = useState(initialPerPage);

  const nextPage = useCallback(() => setPage((p) => p + 1), []);
  const prevPage = useCallback(() => setPage((p) => Math.max(1, p - 1)), []);
  const goToPage = useCallback((p: number) => setPage(Math.max(1, p)), []);
  const resetPage = useCallback(() => setPage(1), []);

  return {
    page,
    perPage,
    setPerPage,
    nextPage,
    prevPage,
    goToPage,
    resetPage,
  };
}
