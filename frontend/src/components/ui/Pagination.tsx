interface PaginationProps {
  page: number;
  totalPages: number;
  onPageChange: (page: number) => void;
}

const MIN_PAGES_FOR_FIRST_AND_LAST = 3;

export function Pagination({
  page,
  totalPages,
  onPageChange,
}: PaginationProps) {
  if (totalPages <= 1) return null;

  const isFirstPage = page <= 1;
  const isLastPage = page >= totalPages;
  const showFirstAndLast = totalPages >= MIN_PAGES_FOR_FIRST_AND_LAST;

  return (
    <nav
      aria-label="Pagination"
      style={{
        display: "flex",
        flexWrap: "wrap",
        gap: "8px",
        alignItems: "center",
        marginTop: "16px",
      }}
    >
      {showFirstAndLast && (
        <button
          type="button"
          onClick={() => onPageChange(1)}
          disabled={isFirstPage}
        >
          First
        </button>
      )}
      <button
        type="button"
        onClick={() => onPageChange(page - 1)}
        disabled={isFirstPage}
      >
        Previous
      </button>
      <span>
        Page {page} of {totalPages}
      </span>
      <button
        type="button"
        onClick={() => onPageChange(page + 1)}
        disabled={isLastPage}
      >
        Next
      </button>
      {showFirstAndLast && (
        <button
          type="button"
          onClick={() => onPageChange(totalPages)}
          disabled={isLastPage}
        >
          Last
        </button>
      )}
    </nav>
  );
}
