import type { ReactNode } from "react";
import { ExternalLinkIcon } from "../icons";

interface ExternalLinkProps {
  href: string;
  children: ReactNode;
}

export function ExternalLink({ href, children }: ExternalLinkProps) {
  return (
    <a href={href} target="_blank" rel="noopener noreferrer">
      {children}
      <ExternalLinkIcon className="external-link-icon" />
      <span className="visually-hidden"> (opens in a new tab)</span>
    </a>
  );
}
