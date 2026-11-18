import { Tabs } from "./Tabs";

const adminTabs = [
  { to: "/admin/teams", label: "Teams" },
  { to: "/admin/users", label: "Users" },
  { to: "/admin/canary", label: "Canary" },
  { to: "/admin/notification-outbox", label: "Notification outbox" },
];

export function AdminNav() {
  return <Tabs label="Admin" items={adminTabs} />;
}
