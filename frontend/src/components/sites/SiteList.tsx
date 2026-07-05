import type { components } from "../../generated/schema";
import { SiteRow } from "./SiteRow.tsx";

type SiteResponse = components["schemas"]["SiteResponse"];

interface SiteListProps {
  sites: SiteResponse[];
  onDelete?: (site: SiteResponse) => void;
}

export function SiteList({ sites, onDelete }: SiteListProps) {
  if (sites.length === 0) {
    return <p>No sites configured. Add one!</p>;
  }
  return (
    <table className="data-table">
      <thead>
        <tr className="table-header-row">
          <th className="table-cell-left">Name</th>
          <th className="table-cell-left">URL</th>
          <th className="table-cell-left">Team</th>
          <th className="table-cell-center">Status</th>
          <th className="table-cell-center">Cert</th>
          <th className="table-cell-right">Latency</th>
          <th className="table-cell-right">Last Checked</th>
          {onDelete && <th className="table-cell-right">Actions</th>}
        </tr>
      </thead>
      <tbody>
        {sites.map((site) => (
          <SiteRow key={site.id} site={site} onDelete={onDelete} />
        ))}
      </tbody>
    </table>
  );
}
