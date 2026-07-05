import { Link, useLocation } from "react-router-dom";

export interface TabItem {
  to: string;
  label: string;
}

interface TabsProps {
  items: TabItem[];
}

export function Tabs({ items }: TabsProps) {
  const location = useLocation();
  return (
    <nav className="tabs-nav">
      {items.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          className={
            location.pathname === item.to ? "tab-link active" : "tab-link"
          }
        >
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
