use crate::models::site::CertStatus;
use crate::probe::cert::CertCheck;
use crate::probe::http::{ProbeResult, UNKNOWN_PROBE_ERROR_MESSAGE};
use chrono::Utc;

pub(super) fn probe_status_code(result: &ProbeResult) -> String {
    result
        .status_code
        .map_or_else(|| "N/A".to_owned(), |status| status.as_u16().to_string())
}

pub(super) fn probe_error(result: &ProbeResult) -> &str {
    result
        .error_message
        .as_deref()
        .unwrap_or(UNKNOWN_PROBE_ERROR_MESSAGE)
}

pub(super) fn cert_summary(cert: &CertCheck) -> String {
    if cert.status == CertStatus::Expired {
        "has EXPIRED".to_owned()
    } else {
        cert.expires_at.map_or_else(
            || "is expiring soon".to_owned(),
            |expires_at| {
                let days = expires_at.signed_duration_since(Utc::now()).num_days();
                format!("expires in {days} day(s)")
            },
        )
    }
}

pub(super) fn cert_expiry(cert: &CertCheck) -> String {
    cert.expires_at.map_or_else(
        || "unknown".to_owned(),
        |expires_at| expires_at.format("%Y-%m-%d %H:%M UTC").to_string(),
    )
}
