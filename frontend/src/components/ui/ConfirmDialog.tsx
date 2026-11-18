import { useLayoutEffect, useRef } from "react";
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

const STOP_WATCHING_FOR_REMOVAL_AFTER_MS = 10_000;

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
  const elementThatOpenedTheDialog = useRef<HTMLElement | null>(null);

  useLayoutEffect(() => {
    if (isOpen) {
      elementThatOpenedTheDialog.current =
        document.activeElement as HTMLElement | null;
    }
  }, [isOpen]);

  function confirm() {
    onConfirm();
    onClose();
    const opener = elementThatOpenedTheDialog.current;
    if (opener) focusPageHeadingIfTheOpenerIsRemoved(opener);
  }

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={title}>
      <p style={{ margin: "0 0 24px 0", color: "var(--color-muted)" }}>
        {message}
      </p>
      <div style={{ display: "flex", gap: "12px", justifyContent: "flex-end" }}>
        <button
          data-autofocus
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
          onClick={confirm}
          style={{
            padding: "8px 16px",
            border: "none",
            borderRadius: "6px",
            background: isDestructive
              ? "var(--color-danger)"
              : "var(--color-primary)",
            color: isDestructive
              ? "var(--color-danger-on)"
              : "var(--color-primary-text)",
            cursor: "pointer",
          }}
        >
          {confirmText}
        </button>
      </div>
    </Modal>
  );
}

function focusPageHeadingIfTheOpenerIsRemoved(opener: HTMLElement) {
  const watchForTheOpenerBeingRemoved = new MutationObserver(() => {
    if (opener.isConnected) return;
    stopWatching();
    const focusWasLost = document.activeElement === document.body;
    if (!focusWasLost) return;
    const pageHeading = document.querySelector<HTMLElement>("main h1");
    if (!pageHeading) return;
    pageHeading.tabIndex = -1;
    pageHeading.focus();
  });
  const giveUpTimer = setTimeout(
    stopWatching,
    STOP_WATCHING_FOR_REMOVAL_AFTER_MS,
  );
  function stopWatching() {
    watchForTheOpenerBeingRemoved.disconnect();
    clearTimeout(giveUpTimer);
  }
  watchForTheOpenerBeingRemoved.observe(document.body, {
    childList: true,
    subtree: true,
  });
}
