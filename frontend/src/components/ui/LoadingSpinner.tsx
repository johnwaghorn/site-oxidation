export function LoadingSpinner() {
  return (
    <div style={{ padding: "20px", textAlign: "center" }}>
      <img
        src="/site-oxidation.svg"
        alt="Loading"
        style={{
          width: "48px",
          height: "48px",
          animation: "spin 1.5s linear infinite",
        }}
      />
      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
