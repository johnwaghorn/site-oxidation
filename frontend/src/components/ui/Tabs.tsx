import { Link, useLocation } from "react-router-dom";
import { tabsNav, activeTab, inactiveTab } from "../../lib/styles";

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
    <nav style={tabsNav}>
      {items.map((item) => (
        <Link
          key={item.to}
          to={item.to}
          style={location.pathname === item.to ? activeTab : inactiveTab}
        >
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
