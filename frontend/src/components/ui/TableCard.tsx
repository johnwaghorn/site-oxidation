import type { ReactNode } from "react";

interface TableCardProps {
  toolbar: ReactNode;
  children: ReactNode;
}

export function TableCard({ toolbar, children }: TableCardProps) {
  return (
    <div className="table-card">
      <div className="table-card-toolbar">{toolbar}</div>
      <div className="table-card-body">{children}</div>
    </div>
  );
}

export function TableCardEmpty({ children }: { children: ReactNode }) {
  return <p className="table-card-empty muted-text">{children}</p>;
}
