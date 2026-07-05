import type { InputHTMLAttributes, SelectHTMLAttributes } from "react";

export function FormInput({
  className,
  ...props
}: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      {...props}
      className={className ? `form-field ${className}` : "form-field"}
    />
  );
}

export function FormSelect({
  className,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      {...props}
      className={className ? `form-field ${className}` : "form-field"}
    />
  );
}
