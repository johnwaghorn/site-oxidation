import { Link } from "react-router-dom";
import type { components } from "../../generated/schema";
import { StatusBadge } from "../ui/StatusBadge";
import { CertBadge } from "../ui/CertBadge";
import { Truncate } from "../ui/Truncate";

type SiteResponse = components["schemas"]["SiteResponse"];

interface SiteRowProps {
  site: SiteResponse;
  onDelete?: (site: SiteResponse) => void;
}

export function SiteRow({ site, onDelete }: SiteRowProps) {
  const lastChecked = site.last_checked_at
    ? new Date(site.last_checked_at).toLocaleString()
    : "Never";

  return (
    <tr className="table-row">
      <td className="table-cell" style={{ fontWeight: 500 }}>
        <Link to={`/sites/${site.id}`}>{site.name}</Link>
      </td>
      <td className="table-cell muted-text" style={{ fontSize: "14px" }}>
        <a href={site.url} target="_blank" rel="noopener noreferrer">
          {site.url}
        </a>
      </td>
      <td className="table-cell muted-text" style={{ fontSize: "14px" }}>
        <Truncate text={site.team_name ?? "No team"} />
      </td>
      <td className="table-cell-center">
        <StatusBadge status={site.status} />
      </td>
      <td className="table-cell-center">
        <CertBadge status={site.cert_status} expiresAt={site.cert_expires_at} />
      </td>
      <td className="table-cell-right" style={{ fontFamily: "monospace" }}>
        {site.last_response_time_ms != null
          ? `${site.last_response_time_ms}ms`
          : "-"}
      </td>
      <td className="table-cell-right muted-text" style={{ fontSize: "14px" }}>
        {lastChecked}
      </td>
      {onDelete && (
        <td className="table-cell-right">
          <button
            className="button-table-action button-table-danger"
            onClick={() => onDelete(site)}
            style={{
              padding: "6px 8px",
            }}
          >
            Delete
          </button>
        </td>
      )}
    </tr>
  );
}
