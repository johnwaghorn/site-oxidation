import type { SVGProps } from "react";
import { IconBase } from "./IconBase";

export function CloseIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <IconBase {...props}>
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </IconBase>
  );
}
