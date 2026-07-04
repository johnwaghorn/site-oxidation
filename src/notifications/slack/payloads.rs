use crate::models::site::{CertStatus, SiteRow};
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct SlackPayload {
    text: String,
}

pub(super) fn site_down(site: &SiteRow, result: &ProbeResult) -> SlackPayload {
    let status = result
        .status_code
        .map_or_else(|| "N/A".to_owned(), |status| status.as_u16().to_string());
    let error = result
        .error_message
        .as_deref()
        .unwrap_or("no error message");
    SlackPayload {
        text: format!(
            ":rotating_light: Site '{}' is DOWN\nURL: {}\nExpected status: {}\nActual status: {}\nError: {}",
            site.name, site.url, site.expected_status, status, error
        ),
    }
}

pub(super) fn site_recovered(site: &SiteRow) -> SlackPayload {
    SlackPayload {
        text: format!(
            ":white_check_mark: Site '{}' is back UP\nURL: {}",
            site.name, site.url
        ),
    }
}

pub(super) fn cert_expiring(site: &SiteRow, cert: &CertCheck) -> SlackPayload {
    let summary = if cert.status == CertStatus::Expired {
        "has EXPIRED".to_owned()
    } else {
        cert.expires_at.map_or_else(
            || "is expiring soon".to_owned(),
            |expires_at| {
                let days = expires_at.signed_duration_since(Utc::now()).num_days();
                format!("expires in {days} day(s)")
            },
        )
    };
    let expiry = cert.expires_at.map_or_else(
        || "unknown".to_owned(),
        |expires_at| expires_at.format("%Y-%m-%d %H:%M UTC").to_string(),
    );
    SlackPayload {
        text: format!(
            ":warning: TLS certificate for site '{}' {}\nURL: {}\nExpires: {}",
            site.name, summary, site.url, expiry
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::site::{SiteRow, SiteStatus};
    use chrono::Duration as ChronoDuration;
    use reqwest::StatusCode;

    fn site_row() -> SiteRow {
        SiteRow {
            id: 1,
            name: "Waghorn Technology Ltd".to_owned(),
            url: "https://waghorn.tech".to_owned(),
            expected_status: 200,
            expected_text: None,
            status: SiteStatus::Up,
            tls_allow_untrusted: false,
            slack_webhook_url: Some("https://hooks.slack.test/services/test".to_owned()),
            cert_status: None,
            notify_site_down: true,
            notify_site_recovered: true,
            notify_cert_expiring: true,
        }
    }

    #[test]
    fn site_down_payload_includes_probe_context() {
        let site = site_row();
        let result = ProbeResult {
            status: SiteStatus::Down,
            status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            latency_ms: Some(120),
            error_message: Some("Server is cooked".to_owned()),
        };
        let payload = site_down(&site, &result);
        assert!(payload.text.contains("Waghorn Technology Ltd"));
        assert!(payload.text.contains("https://waghorn.tech"));
        assert!(payload.text.contains("Expected status: 200"));
        assert!(payload.text.contains("Actual status: 500"));
        assert!(payload.text.contains("Server is cooked"));
    }

    #[test]
    fn site_recovered_payload_includes_site_context() {
        let site = site_row();
        let payload = site_recovered(&site);
        assert!(payload.text.contains("Waghorn Technology Ltd"));
        assert!(payload.text.contains("https://waghorn.tech"));
        assert!(payload.text.contains("back UP"));
    }

    #[test]
    fn cert_expiring_payload_includes_days_remaining() {
        let site = site_row();
        let cert = CertCheck {
            status: CertStatus::Expiring,
            expires_at: Some(Utc::now() + ChronoDuration::days(10)),
        };
        let payload = cert_expiring(&site, &cert);
        assert!(payload.text.contains("Waghorn Technology Ltd"));
        assert!(
            payload.text.contains("expires in 9 day(s)")
                || payload.text.contains("expires in 10 day(s)")
        );
    }

    #[test]
    fn cert_expiring_payload_reports_expired_certs() {
        let site = site_row();
        let cert = CertCheck {
            status: CertStatus::Expired,
            expires_at: Some(Utc::now() - ChronoDuration::days(1)),
        };
        let payload = cert_expiring(&site, &cert);
        assert!(payload.text.contains("has EXPIRED"));
    }
}
