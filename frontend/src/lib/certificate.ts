import type { components } from "../generated/schema";

type CertStatus = components["schemas"]["CertStatus"];

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
      const expiry = new Date(expiresAt);
      const days = Math.ceil((expiry.getTime() - Date.now()) / 86400000);
      return `Valid until ${expiry.toLocaleDateString()} (${days} day${
        days === 1 ? "" : "s"
      })`;
    }
  }
}
