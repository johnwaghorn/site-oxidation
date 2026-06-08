interface ErrorMessageProps {
  error: Error | null;
}

export function ErrorMessage({ error }: ErrorMessageProps) {
  if (!error) return null;
  return (
    <div
      style={{
        marginTop: "16px",
        padding: "16px",
        backgroundColor: "var(--color-danger-bg)",
        color: "var(--color-danger-text)",
        border: "1px solid var(--color-danger)",
        borderRadius: "10px",
      }}
    >
      Error: {error.message || "Something went wrong. Please try again."}
    </div>
  );
}
