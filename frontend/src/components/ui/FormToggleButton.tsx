import { compactInput } from "../../lib/styles";

interface FormToggleButtonProps {
  isOpen: boolean;
  openLabel: string;
  onClick: () => void;
}

export function FormToggleButton({
  isOpen,
  openLabel,
  onClick,
}: FormToggleButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{ ...compactInput, padding: "10px 14px" }}
    >
      {isOpen ? "Close Form" : openLabel}
    </button>
  );
}
