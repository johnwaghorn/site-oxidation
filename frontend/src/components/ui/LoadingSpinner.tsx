export function LoadingSpinner() {
  return (
    <div className="loading-spinner" role="status">
      <div className="loading-spinner-logo-wrap">
        <img
          className="loading-spinner-logo"
          src="/site-oxidation.svg"
          alt=""
        />
        <span
          className="loading-spinner-drip loading-spinner-drip-a"
          aria-hidden="true"
        />
        <span
          className="loading-spinner-drip loading-spinner-drip-b"
          aria-hidden="true"
        />
      </div>
      <span className="loading-spinner-label">Loading</span>
    </div>
  );
}
