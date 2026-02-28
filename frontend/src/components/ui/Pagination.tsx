interface PaginationProps {
    page: number
    totalPages: number
    onPageChange: (page:number) => void
}

export function Pagination({page, totalPages, onPageChange}: PaginationProps) {
    if (totalPages <= 1) return null

    return (
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', marginTop: '16px' }}>
            <button onClick={() => onPageChange(page - 1)} disabled={page <= 1}>
                Previous
            </button>
            <span>
        Page {page} of {totalPages}
      </span>
            <button onClick={() => onPageChange(page + 1)} disabled={page >= totalPages}>
                Next
            </button>
        </div>
    )
}
