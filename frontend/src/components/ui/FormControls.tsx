import {
  useState,
  type InputHTMLAttributes,
  type SelectHTMLAttributes,
} from "react";
import { EyeIcon, EyeOffIcon } from "../icons";

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

export function SecretInput({
  className,
  style,
  ...props
}: InputHTMLAttributes<HTMLInputElement>) {
  const [isRevealed, setIsRevealed] = useState(false);

  return (
    <div
      className={className ? `form-field ${className}` : "form-field"}
      style={{ display: "flex", alignItems: "center", padding: 0, ...style }}
    >
      <input
        autoComplete="new-password"
        {...props}
        type={isRevealed ? "text" : "password"}
        className="form-field-inner"
      />
      <button
        type="button"
        className="form-field-button"
        aria-label={isRevealed ? "Hide value" : "Show value"}
        aria-pressed={isRevealed}
        onClick={() => setIsRevealed((revealed) => !revealed)}
      >
        {isRevealed ? <EyeOffIcon /> : <EyeIcon />}
      </button>
    </div>
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
