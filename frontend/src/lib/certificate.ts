import type { components } from "../generated/schema";

type CertStatus = components["schemas"]["CertStatus"];

const MILLISECONDS_IN_A_DAY = 24 * 60 * 60 * 1000;

export function describeCertificate(
  status: CertStatus | null | undefined,
  expiresAt?: string | null,
): string {
  if (status == null)
    return "Certificate status unavailable - not checked yet, blocked, or the certificate could not be read";
  switch (status) {
    case "none":
      return "Plain HTTP - no certificate";
    case "invalid":
      return "Invalid certificate: untrusted, hostname mismatch, or unreachable";
    case "expired":
      return expiresAt
        ? `Expired on ${new Date(expiresAt).toLocaleDateString()}`
        : "Certificate expired";
    default: {
      if (!expiresAt) return "Certificate valid";
      return `Valid until ${formatExpiryDate(expiresAt)} (${formatDaysLeft(expiresAt)})`;
    }
  }
}

export function summariseCertificate(
  status: CertStatus | null | undefined,
  expiresAt?: string | null,
): { headline: string; detail: string } {
  if (status == null)
    return {
      headline: "Unknown",
      detail: "Not checked yet, blocked, or could not be read",
    };
  switch (status) {
    case "none":
      return { headline: "No TLS", detail: "Plain HTTP - no certificate" };
    case "invalid":
      return {
        headline: "Invalid",
        detail: "Untrusted, hostname mismatch, or unreachable",
      };
    case "expired":
      return {
        headline: "Expired",
        detail: expiresAt
          ? `Expired on ${formatExpiryDate(expiresAt)}`
          : "The certificate has expired",
      };
    default:
      return expiresAt
        ? {
            headline: formatDaysLeft(expiresAt),
            detail: `Valid until ${formatExpiryDate(expiresAt)}`,
          }
        : { headline: "Valid", detail: "Expiry date unknown" };
  }
}

function formatExpiryDate(expiresAt: string) {
  return new Date(expiresAt).toLocaleDateString();
}

function formatDaysLeft(expiresAt: string) {
  const days = Math.ceil(
    (new Date(expiresAt).getTime() - Date.now()) / MILLISECONDS_IN_A_DAY,
  );
  return `${days} day${days === 1 ? "" : "s"}`;
}
