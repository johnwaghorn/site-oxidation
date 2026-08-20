import { Tabs } from "./Tabs";

const adminTabs = [
  { to: "/admin/teams", label: "Teams" },
  { to: "/admin/users", label: "Users" },
  { to: "/admin/canary", label: "Canary" },
];

export function AdminNav() {
  return <Tabs items={adminTabs} />;
}
