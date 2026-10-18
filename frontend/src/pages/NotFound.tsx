import { useLocation, useNavigate } from "react-router-dom";
import { StatusBadge } from "../components/ui/StatusBadge";

export function NotFound() {
  const location = useLocation();
  const navigate = useNavigate();

  return (
    <div className="page-wrapper not-found">
      <div className="not-found-code" aria-hidden="true">
        404
      </div>
      <h1 className="page-title">Page not found</h1>
      <div className="not-found-probe">
        <StatusBadge status="down" />
        <code className="not-found-path">{location.pathname}</code>
        <span className="muted-text">Not Found</span>
      </div>
      <p className="muted-text" style={{ maxWidth: "440px", margin: 0 }}>
        We checked, and there's nothing at this address. The link may be out of
        date, or the page may have moved.
      </p>
      <button
        className="button-primary-action"
        onClick={() => void navigate("/")}
      >
        Go to your sites
      </button>
    </div>
  );
}
