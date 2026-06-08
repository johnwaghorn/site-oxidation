import { Modal } from "./Modal";

interface ConfirmDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  isDestructive?: boolean;
}

export function ConfirmDialog({
  isOpen,
  onClose,
  onConfirm,
  title,
  message,
  confirmText = "Confirm",
  cancelText = "Cancel",
  isDestructive = false,
}: ConfirmDialogProps) {
  return (
    <Modal isOpen={isOpen} onClose={onClose} title={title}>
      <p style={{ margin: "0 0 24px 0", color: "var(--color-muted)" }}>
        {message}
      </p>
      <div style={{ display: "flex", gap: "12px", justifyContent: "flex-end" }}>
        <button
          onClick={onClose}
          style={{
            padding: "8px 16px",
            border: "1px solid var(--color-border)",
            borderRadius: "6px",
            background: "var(--color-surface)",
            color: "var(--color-text)",
            cursor: "pointer",
          }}
        >
          {cancelText}
        </button>
        <button
          onClick={() => {
            onConfirm();
            onClose();
          }}
          style={{
            padding: "8px 16px",
            border: "none",
            borderRadius: "6px",
            background: isDestructive
              ? "var(--color-danger)"
              : "var(--color-primary)",
            color: isDestructive ? "#ffffff" : "var(--color-primary-text)",
            cursor: "pointer",
          }}
        >
          {confirmText}
        </button>
      </div>
    </Modal>
  );
}
