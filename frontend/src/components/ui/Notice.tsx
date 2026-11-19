import type { ReactNode } from "react";
import { WarningIcon } from "../icons";

interface NoticeProps {
  tone: "danger" | "warning" | "neutral";
  action?: ReactNode;
  children: ReactNode;
}

export function Notice({ tone, action, children }: NoticeProps) {
  return (
    <div className={`notice is-${tone}`}>
      <WarningIcon className="notice-icon" width="20" height="20" />
      <p className="notice-message">{children}</p>
      {action}
    </div>
  );
}
