import { Link } from "react-router-dom";
import type { components } from "../../generated/schema";
import {
  tableRow,
  tableCell,
  tableCellCenter,
  tableCellRight,
  mutedText,
} from "../../lib/styles";
import { StatusBadge } from "../ui/StatusBadge";
import { CertBadge } from "../ui/CertBadge";

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
    <tr style={tableRow}>
      <td style={{ ...tableCell, fontWeight: 500 }}>
        <Link to={`/sites/${site.id}`}>{site.name}</Link>
      </td>
      <td style={{ ...tableCell, ...mutedText, fontSize: "14px" }}>
        <a href={site.url} target="_blank" rel="noopener noreferrer">
          {site.url}
        </a>
      </td>
      <td style={{ ...tableCell, ...mutedText, fontSize: "14px" }}>
        {site.team_name ?? "No team"}
      </td>
      <td style={tableCellCenter}>
        <StatusBadge status={site.status} />
      </td>
      <td style={tableCellCenter}>
        <CertBadge status={site.cert_status} expiresAt={site.cert_expires_at} />
      </td>
      <td style={{ ...tableCellRight, fontFamily: "monospace" }}>
        {site.last_response_time_ms != null
          ? `${site.last_response_time_ms}ms`
          : "-"}
      </td>
      <td style={{ ...tableCellRight, ...mutedText, fontSize: "14px" }}>
        {lastChecked}
      </td>
      {onDelete && (
        <td style={tableCellRight}>
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
