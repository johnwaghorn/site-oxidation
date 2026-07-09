use crate::models::site::SiteRow;
use crate::notifications::TEST_MESSAGE_TITLE;
use crate::notifications::format;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;

pub(super) fn test_subject() -> &'static str {
    TEST_MESSAGE_TITLE
}

pub(super) fn test_body(triggered_by: &str) -> String {
    format!(
        "This is a test message from Site Oxidation, triggered by {}.\n{}\n",
        triggered_by,
        env!("CARGO_PKG_REPOSITORY")
    )
}

pub(super) fn site_down_subject(site: &SiteRow) -> String {
    format!("Site '{}' is DOWN", site.name)
}

pub(super) fn site_down_body(site: &SiteRow, result: &ProbeResult) -> String {
    format!(
        "Site '{}' is DOWN\nURL: {}\nExpected status: {}\nActual status: {}\nError: {}\n",
        site.name,
        site.url,
        site.expected_status,
        format::probe_status_code(result),
        format::probe_error(result)
    )
}

pub(super) fn site_recovered_subject(site: &SiteRow) -> String {
    format!("Site '{}' is back UP", site.name)
}

pub(super) fn site_recovered_body(site: &SiteRow) -> String {
    format!("Site '{}' is back UP\nURL: {}\n", site.name, site.url)
}

pub(super) fn cert_expiring_subject(site: &SiteRow, cert: &CertCheck) -> String {
    format!(
        "TLS certificate for site '{}' {}",
        site.name,
        format::cert_summary(cert)
    )
}

pub(super) fn cert_expiring_body(site: &SiteRow, cert: &CertCheck) -> String {
    format!(
        "TLS certificate for site '{}' {}\nURL: {}\nExpires: {}\n",
        site.name,
        format::cert_summary(cert),
        site.url,
        format::cert_expiry(cert)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notifications::TeamNotificationConfig;
    use crate::models::site::{CertStatus, SiteStatus};
    use chrono::{Duration as ChronoDuration, Utc};
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
            cert_status: None,
            notifications: TeamNotificationConfig::default(),
        }
    }

    #[test]
    fn site_down_email_includes_probe_context() {
        let site = site_row();
        let result = ProbeResult {
            status: SiteStatus::Down,
            status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            latency_ms: Some(120),
            error_message: Some("Server is cooked".to_owned()),
        };
        assert_eq!(
            site_down_subject(&site),
            "Site 'Waghorn Technology Ltd' is DOWN"
        );
        let body = site_down_body(&site, &result);
        assert!(body.contains("https://waghorn.tech"));
        assert!(body.contains("Expected status: 200"));
        assert!(body.contains("Actual status: 500"));
        assert!(body.contains("Server is cooked"));
    }

    #[test]
    fn site_recovered_email_includes_site_context() {
        let site = site_row();
        assert_eq!(
            site_recovered_subject(&site),
            "Site 'Waghorn Technology Ltd' is back UP"
        );
        assert!(site_recovered_body(&site).contains("https://waghorn.tech"));
    }

    #[test]
    fn cert_expiring_email_includes_days_remaining() {
        let site = site_row();
        let cert = CertCheck {
            status: CertStatus::Expiring,
            expires_at: Some(Utc::now() + ChronoDuration::days(10)),
        };
        let subject = cert_expiring_subject(&site, &cert);
        assert!(subject.contains("TLS certificate for site 'Waghorn Technology Ltd'"));
        assert!(
            subject.contains("expires in 9 day(s)") || subject.contains("expires in 10 day(s)")
        );
        let body = cert_expiring_body(&site, &cert);
        assert!(body.contains("https://waghorn.tech"));
        assert!(body.contains("Expires:"));
    }
}
