import { Link, useLocation } from "react-router-dom";

export interface TabItem {
  to: string;
  label: string;
}

interface TabsProps {
  label: string;
  items: TabItem[];
}

export function Tabs({ label, items }: TabsProps) {
  const location = useLocation();
  return (
    <nav className="tabs-nav" aria-label={label}>
      {items.map((item) => {
        const isActive = location.pathname === item.to;
        return (
          <Link
            key={item.to}
            to={item.to}
            className={isActive ? "tab-link active" : "tab-link"}
            aria-current={isActive ? "page" : undefined}
          >
            {item.label}
          </Link>
        );
      })}
    </nav>
  );
}
