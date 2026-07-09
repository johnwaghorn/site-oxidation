use crate::models::site::SiteRow;
use crate::notifications::TEST_MESSAGE_TITLE;
use crate::notifications::format;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use serde_json::{Value, json};

pub(super) fn test(triggered_by: &str) -> Value {
    card(
        TEST_MESSAGE_TITLE,
        "Good",
        &json!([
            { "title": "Triggered by", "value": triggered_by },
            { "title": "Source", "value": env!("CARGO_PKG_REPOSITORY") },
        ]),
    )
}

// Microsoft Teams workflow webhooks expect an Adaptive Card
pub(super) fn site_down(site: &SiteRow, result: &ProbeResult) -> Value {
    card(
        &format!("Site '{}' is DOWN", site.name),
        "Attention",
        &json!([
            { "title": "URL", "value": site.url },
            { "title": "Expected status", "value": site.expected_status.to_string() },
            { "title": "Actual status", "value": format::probe_status_code(result) },
            { "title": "Error", "value": format::probe_error(result) },
        ]),
    )
}

pub(super) fn site_recovered(site: &SiteRow) -> Value {
    card(
        &format!("Site '{}' is back UP", site.name),
        "Good",
        &json!([{ "title": "URL", "value": site.url }]),
    )
}

pub(super) fn cert_expiring(site: &SiteRow, cert: &CertCheck) -> Value {
    card(
        &format!(
            "TLS certificate for site '{}' {}",
            site.name,
            format::cert_summary(cert)
        ),
        "Warning",
        &json!([
            { "title": "URL", "value": site.url },
            { "title": "Expires", "value": format::cert_expiry(cert) },
        ]),
    )
}

fn card(title: &str, color: &str, facts: &Value) -> Value {
    json!({
        "type": "message",
        "attachments": [{
            "contentType": "application/vnd.microsoft.card.adaptive",
            "content": {
                "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
                "type": "AdaptiveCard",
                "version": "1.4",
                "body": [
                    {
                        "type": "TextBlock",
                        "text": title,
                        "weight": "Bolder",
                        "size": "Medium",
                        "color": color,
                        "wrap": true
                    },
                    {
                        "type": "FactSet",
                        "facts": facts
                    }
                ]
            }
        }]
    })
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
            notifications: TeamNotificationConfig {
                microsoft_teams_webhook_url: Some("https://teams.waghorn.tech/webhook".to_owned()),
                ..TeamNotificationConfig::default()
            },
        }
    }

    #[test]
    fn site_down_card_includes_probe_context() {
        let site = site_row();
        let result = ProbeResult {
            status: SiteStatus::Down,
            status_code: Some(StatusCode::INTERNAL_SERVER_ERROR),
            latency_ms: Some(120),
            error_message: Some("Server is cooked".to_owned()),
        };
        let rendered = site_down(&site, &result).to_string();
        assert!(rendered.contains("application/vnd.microsoft.card.adaptive"));
        assert!(rendered.contains("Site 'Waghorn Technology Ltd' is DOWN"));
        assert!(rendered.contains("https://waghorn.tech"));
        assert!(rendered.contains("500"));
        assert!(rendered.contains("Server is cooked"));
    }

    #[test]
    fn site_recovered_card_includes_site_context() {
        let rendered = site_recovered(&site_row()).to_string();
        assert!(rendered.contains("Site 'Waghorn Technology Ltd' is back UP"));
        assert!(rendered.contains("https://waghorn.tech"));
    }

    #[test]
    fn cert_expiring_card_includes_days_remaining() {
        let cert = CertCheck {
            status: CertStatus::Expiring,
            expires_at: Some(Utc::now() + ChronoDuration::days(10)),
        };
        let rendered = cert_expiring(&site_row(), &cert).to_string();
        assert!(rendered.contains("TLS certificate for site 'Waghorn Technology Ltd'"));
        assert!(
            rendered.contains("expires in 9 day(s)") || rendered.contains("expires in 10 day(s)")
        );
    }
}
