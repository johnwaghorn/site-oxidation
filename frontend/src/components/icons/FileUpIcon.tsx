import type { SVGProps } from "react";
import { IconBase } from "./IconBase";

export function FileUpIcon(props: SVGProps<SVGSVGElement>) {
  return (
    <IconBase {...props}>
      <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z" />
      <path d="M14 3v5h5" />
      <path d="M12 12v6" />
      <path d="m9 15 3-3 3 3" />
    </IconBase>
  );
}
