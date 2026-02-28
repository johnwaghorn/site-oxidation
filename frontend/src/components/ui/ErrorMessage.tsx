interface ErrorMessageProps {
    error: Error | null
}

export function ErrorMessage({error}: ErrorMessageProps) {
    if (!error) return null
    return (
        <div style={{ padding: '16px', backgroundColor: '#fee2e2', color: '#991b1b', borderRadius: '4px' }}>
            Error: {error.message}
        </div>
    )
}
