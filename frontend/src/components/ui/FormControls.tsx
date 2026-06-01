import {
  useState,
  type InputHTMLAttributes,
  type SelectHTMLAttributes,
} from "react";
import { polishedFieldChrome, polishedFieldFocus } from "../../lib/styles";

export function FormInput({
  style,
  onFocus,
  onBlur,
  ...props
}: InputHTMLAttributes<HTMLInputElement>) {
  const [isFocused, setIsFocused] = useState(false);

  return (
    <input
      {...props}
      onFocus={(e) => {
        setIsFocused(true);
        onFocus?.(e);
      }}
      onBlur={(e) => {
        setIsFocused(false);
        onBlur?.(e);
      }}
      style={{
        ...polishedFieldChrome,
        ...(isFocused ? polishedFieldFocus : null),
        ...style,
      }}
    />
  );
}

export function FormSelect({
  style,
  onFocus,
  onBlur,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement>) {
  const [isFocused, setIsFocused] = useState(false);

  return (
    <select
      {...props}
      onFocus={(e) => {
        setIsFocused(true);
        onFocus?.(e);
      }}
      onBlur={(e) => {
        setIsFocused(false);
        onBlur?.(e);
      }}
      style={{
        ...polishedFieldChrome,
        ...(isFocused ? polishedFieldFocus : null),
        ...style,
      }}
    />
  );
}
