import type { components } from "../../generated/schema";
import {
  table,
  tableHeaderRow,
  tableCellLeft,
  tableCellCenter,
  tableCellRight,
  tableCell,
} from "../../lib/styles";
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
    <table style={table}>
      <thead>
        <tr style={tableHeaderRow}>
          <th style={tableCellLeft}>Name</th>
          <th style={tableCellLeft}>URL</th>
          <th style={tableCellCenter}>Status</th>
          <th style={tableCellRight}>Latency</th>
          <th style={tableCellRight}>Last Checked</th>
          {onDelete && <th style={tableCell}></th>}
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
