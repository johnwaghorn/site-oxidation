import { pageTitle, pageWrapper, mutedText } from "../lib/styles";

export function Notifications() {
  return (
    <div style={pageWrapper}>
      <h1 style={pageTitle}>Notifications</h1>
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
          Notification settings are on the way
        </h2>
        <p style={{ ...mutedText, maxWidth: "520px", margin: 0 }}>
          This will be the place to configure alerts for outages, recoveries,
          and certificate events.
        </p>
      </section>
    </div>
  );
}
