import { pageTitle, pageWrapper, mutedText } from "../lib/styles";

export function Backups() {
  return (
    <div style={pageWrapper}>
      <h1 style={pageTitle}>Backups & Restores</h1>
      <section
        style={{
          maxWidth: "680px",
          padding: "28px",
          border: "1px solid var(--color-border)",
          borderRadius: "18px",
          background:
            "linear-gradient(135deg, var(--color-primary-soft), transparent 52%), var(--color-surface)",
          boxShadow: "var(--shadow-card)",
        }}
      >
        <p
          style={{
            ...mutedText,
            margin: "0 0 8px 0",
            fontSize: "13px",
            fontWeight: 700,
            letterSpacing: "0.05em",
            textTransform: "uppercase",
          }}
        >
          Coming soon
        </p>
        <h2 style={{ margin: "0 0 10px 0", fontSize: "28px", lineHeight: 1.2 }}>
          Backup and restore tools are on the way
        </h2>
        <p style={{ ...mutedText, maxWidth: "520px", margin: 0 }}>
          This will be the place to export your Site Oxidation data and restore
          it when you need to recover an instance.
        </p>
      </section>
    </div>
  );
}
