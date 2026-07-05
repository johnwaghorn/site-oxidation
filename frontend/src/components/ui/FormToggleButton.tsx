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
      className={isOpen ? "button-secondary-action" : "button-primary-action"}
      onClick={onClick}
      style={{ padding: "10px 14px" }}
    >
      {isOpen ? "Close Form" : openLabel}
    </button>
  );
}
