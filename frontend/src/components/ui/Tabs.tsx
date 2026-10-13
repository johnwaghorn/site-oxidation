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
