mod slack;

use crate::models::site::SiteRow;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use reqwest::Client;

#[derive(Clone)]
pub struct Notifier {
    client: Option<Client>,
}

impl Notifier {
    #[cfg(test)]
    pub fn disabled() -> Self {
        Self { client: None }
    }

    pub fn new(client: Client) -> Self {
        Self {
            client: Some(client),
        }
    }

    pub async fn site_down(&self, site: &SiteRow, result: &ProbeResult) {
        let Some(client) = &self.client else {
            return;
        };
        if !site.notify_site_down {
            return;
        }
        if let Some(webhook_url) = &site.slack_webhook_url {
            slack::site_down(client, webhook_url, site, result).await;
        }
    }

    pub async fn site_recovered(&self, site: &SiteRow) {
        let Some(client) = &self.client else {
            return;
        };
        if !site.notify_site_recovered {
            return;
        }
        if let Some(webhook_url) = &site.slack_webhook_url {
            slack::site_recovered(client, webhook_url, site).await;
        }
    }

    pub async fn cert_expiring(&self, site: &SiteRow, cert: &CertCheck) {
        let Some(client) = &self.client else {
            return;
        };
        if !site.notify_cert_expiring {
            return;
        }
        if let Some(webhook_url) = &site.slack_webhook_url {
            slack::cert_expiring(client, webhook_url, site, cert).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::site::{CertStatus, SiteStatus};
    use crate::tests::TestHttpServer;
    use chrono::{Duration as ChronoDuration, Utc};

    fn site_row(webhook_url: Option<String>) -> SiteRow {
        SiteRow {
            id: 1,
            name: "Waghorn Technology Ltd".to_owned(),
            url: "https://waghorn.tech".to_owned(),
            expected_status: 200,
            expected_text: None,
            status: SiteStatus::Up,
            tls_allow_untrusted: false,
            slack_webhook_url: webhook_url,
            cert_status: None,
            notify_site_down: true,
            notify_site_recovered: true,
            notify_cert_expiring: true,
        }
    }

    fn down_result() -> ProbeResult {
        ProbeResult {
            status: SiteStatus::Down,
            status_code: None,
            latency_ms: None,
            error_message: Some("connection refused".to_owned()),
        }
    }

    fn expiring_cert() -> CertCheck {
        CertCheck {
            status: CertStatus::Expiring,
            expires_at: Some(Utc::now() + ChronoDuration::days(10)),
        }
    }

    #[tokio::test]
    async fn test_site_down_posts_payload_to_webhook() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let site = site_row(Some(format!("{}/webhook", server.base_url())));
        notifier.site_down(&site, &down_result()).await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("POST /webhook"));
        assert!(request.contains("Waghorn Technology Ltd"));
        assert!(request.contains("is DOWN"));
        assert!(request.contains("connection refused"));
    }

    #[tokio::test]
    async fn test_site_down_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let mut site = site_row(Some(format!("{}/webhook", server.base_url())));
        site.notify_site_down = false;
        notifier.site_down(&site, &down_result()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_site_recovered_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let mut site = site_row(Some(format!("{}/webhook", server.base_url())));
        site.notify_site_recovered = false;
        notifier.site_recovered(&site).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_cert_expiring_posts_payload_to_webhook() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let site = site_row(Some(format!("{}/webhook", server.base_url())));
        notifier.cert_expiring(&site, &expiring_cert()).await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("TLS certificate"));
        assert!(request.contains("Waghorn Technology Ltd"));
    }

    #[tokio::test]
    async fn test_cert_expiring_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let mut site = site_row(Some(format!("{}/webhook", server.base_url())));
        site.notify_cert_expiring = false;
        notifier.cert_expiring(&site, &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_disabled_notifier_never_sends() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::disabled();
        let site = site_row(Some(format!("{}/webhook", server.base_url())));
        notifier.site_down(&site, &down_result()).await;
        notifier.site_recovered(&site).await;
        notifier.cert_expiring(&site, &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_no_webhook_means_no_send() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new());
        let site = site_row(None);
        notifier.site_down(&site, &down_result()).await;
        notifier.site_recovered(&site).await;
        notifier.cert_expiring(&site, &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }
}
