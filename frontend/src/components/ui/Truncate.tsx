import type { CSSProperties } from "react";

interface TruncateProps {
  text: string;
  maxWidth?: CSSProperties["maxWidth"];
  style?: CSSProperties;
}

export function Truncate({ text, maxWidth = 200, style }: TruncateProps) {
  return (
    <span
      title={text}
      style={{
        display: "inline-block",
        maxWidth,
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
        verticalAlign: "bottom",
        ...style,
      }}
    >
      {text}
    </span>
  );
}
