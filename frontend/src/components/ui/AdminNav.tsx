import { Tabs } from "./Tabs";

const adminTabs = [
  { to: "/admin/teams", label: "Teams" },
  { to: "/admin/users", label: "Users" },
];

export function AdminNav() {
  return <Tabs items={adminTabs} />;
}
