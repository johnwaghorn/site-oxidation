mod email;
mod format;
mod slack;
mod teams;
mod webhook;

use crate::models::site::SiteRow;
use crate::models::smtp::SmtpSettings;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use futures::future::{BoxFuture, join_all};
use reqwest::Client;
use std::collections::HashSet;

pub(crate) const TEST_MESSAGE_TITLE: &str = "Testing Site Oxidation notification configuration";

#[derive(Clone)]
pub struct Notifier {
    client: Client,
    enabled: bool,
    allow_private_smtp_hosts: bool,
}

impl Notifier {
    pub fn new(client: Client, allow_private_smtp_hosts: bool) -> Self {
        Self {
            client,
            enabled: true,
            allow_private_smtp_hosts,
        }
    }

    pub async fn test_email(&self, smtp: &SmtpSettings, triggered_by: &str) -> Result<(), String> {
        if !self.enabled {
            return Err("Notification sending is disabled".to_owned());
        }
        email::test(smtp, triggered_by, self.allow_private_smtp_hosts).await
    }

    pub async fn test_slack(&self, webhook_url: &str, triggered_by: &str) -> Result<(), String> {
        if !self.enabled {
            return Err("Notification sending is disabled".to_owned());
        }
        slack::test(&self.client, webhook_url, triggered_by).await
    }

    pub async fn test_teams(&self, webhook_url: &str, triggered_by: &str) -> Result<(), String> {
        if !self.enabled {
            return Err("Notification sending is disabled".to_owned());
        }
        teams::test(&self.client, webhook_url, triggered_by).await
    }

    pub async fn site_down(&self, sites: &[&SiteRow], result: &ProbeResult) {
        if !self.enabled {
            return;
        }
        let mut sends: Vec<BoxFuture<'_, ()>> = Vec::new();
        let mut targets = SeenTargets::default();
        for &site in sites {
            if !site.notifications.notify_site_down {
                continue;
            }
            if let Some(url) = site.notifications.slack_webhook_url.as_deref()
                && targets.slack_urls.insert(url)
            {
                sends.push(Box::pin(slack::site_down(&self.client, url, site, result)));
            }
            if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
                && targets.teams_urls.insert(url)
            {
                sends.push(Box::pin(teams::site_down(&self.client, url, site, result)));
            }
            if targets.first_smtp_config(site) {
                sends.push(Box::pin(email::site_down(
                    &site.notifications.smtp,
                    site,
                    result,
                    self.allow_private_smtp_hosts,
                )));
            }
        }
        join_all(sends).await;
    }

    pub async fn site_recovered(&self, sites: &[&SiteRow]) {
        if !self.enabled {
            return;
        }
        let mut sends: Vec<BoxFuture<'_, ()>> = Vec::new();
        let mut targets = SeenTargets::default();
        for &site in sites {
            if !site.notifications.notify_site_recovered {
                continue;
            }
            if let Some(url) = site.notifications.slack_webhook_url.as_deref()
                && targets.slack_urls.insert(url)
            {
                sends.push(Box::pin(slack::site_recovered(&self.client, url, site)));
            }
            if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
                && targets.teams_urls.insert(url)
            {
                sends.push(Box::pin(teams::site_recovered(&self.client, url, site)));
            }
            if targets.first_smtp_config(site) {
                sends.push(Box::pin(email::site_recovered(
                    &site.notifications.smtp,
                    site,
                    self.allow_private_smtp_hosts,
                )));
            }
        }
        join_all(sends).await;
    }

    pub async fn cert_expiring(&self, sites: &[&SiteRow], cert: &CertCheck) {
        if !self.enabled {
            return;
        }
        let mut sends: Vec<BoxFuture<'_, ()>> = Vec::new();
        let mut targets = SeenTargets::default();
        for &site in sites {
            if !site.notifications.notify_cert_expiring {
                continue;
            }
            if let Some(url) = site.notifications.slack_webhook_url.as_deref()
                && targets.slack_urls.insert(url)
            {
                sends.push(Box::pin(slack::cert_expiring(
                    &self.client,
                    url,
                    site,
                    cert,
                )));
            }
            if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
                && targets.teams_urls.insert(url)
            {
                sends.push(Box::pin(teams::cert_expiring(
                    &self.client,
                    url,
                    site,
                    cert,
                )));
            }
            if targets.first_smtp_config(site) {
                sends.push(Box::pin(email::cert_expiring(
                    &site.notifications.smtp,
                    site,
                    cert,
                    self.allow_private_smtp_hosts,
                )));
            }
        }
        join_all(sends).await;
    }
}

#[derive(Default)]
struct SeenTargets<'a> {
    slack_urls: HashSet<&'a str>,
    teams_urls: HashSet<&'a str>,
    smtp_configs: HashSet<&'a SmtpSettings>,
}

impl<'a> SeenTargets<'a> {
    fn first_smtp_config(&mut self, site: &'a SiteRow) -> bool {
        site.notifications.smtp.has_delivery_addresses()
            && self.smtp_configs.insert(&site.notifications.smtp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notifications::TeamNotificationConfig;
    use crate::models::site::{CertStatus, SiteStatus};
    use crate::models::smtp::SmtpSettings;
    use crate::tests::TestHttpServer;
    use chrono::{Duration as ChronoDuration, Utc};

    impl Notifier {
        pub fn disabled() -> Self {
            Self {
                client: Client::new(),
                enabled: false,
                allow_private_smtp_hosts: false,
            }
        }
    }

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
    async fn test_site_down_posts_text_payload_to_slack_webhook() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        notifier.site_down(&[&site], &down_result()).await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("POST /slack"));
        assert!(request.contains("Waghorn Technology Ltd"));
        assert!(request.contains("is DOWN"));
        assert!(request.contains("connection refused"));
    }

    #[tokio::test]
    async fn test_site_down_posts_adaptive_card_to_teams_webhook() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        notifier.site_down(&[&site], &down_result()).await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("POST /teams"));
        assert!(request.contains("application/vnd.microsoft.card.adaptive"));
        assert!(request.contains("Waghorn Technology Ltd"));
        assert!(request.contains("is DOWN"));
    }

    #[tokio::test]
    async fn test_site_down_posts_to_both_channels_when_configured() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        notifier.site_down(&[&site], &down_result()).await;
        assert_eq!(server.request_count(), 2);
    }

    #[tokio::test]
    async fn test_site_down_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        site.notifications.notify_site_down = false;
        notifier.site_down(&[&site], &down_result()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_site_recovered_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        site.notifications.notify_site_recovered = false;
        notifier.site_recovered(&[&site]).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_cert_expiring_posts_to_slack_webhook() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        notifier.cert_expiring(&[&site], &expiring_cert()).await;
        assert_eq!(server.request_count(), 1);
        let request = server.last_request().unwrap();
        assert!(request.contains("POST /slack"));
        assert!(request.contains("TLS certificate"));
        assert!(request.contains("Waghorn Technology Ltd"));
    }

    #[tokio::test]
    async fn test_cert_expiring_respects_notify_flag() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        site.notifications.notify_cert_expiring = false;
        notifier.cert_expiring(&[&site], &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_disabled_notifier_never_sends() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::disabled();
        let mut site = site_row();
        site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        site.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams", server.base_url()));
        notifier.site_down(&[&site], &down_result()).await;
        notifier.site_recovered(&[&site]).await;
        notifier.cert_expiring(&[&site], &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }

    #[tokio::test]
    async fn test_no_webhooks_means_no_send() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let site = site_row();
        notifier.site_down(&[&site], &down_result()).await;
        notifier.site_recovered(&[&site]).await;
        notifier.cert_expiring(&[&site], &expiring_cert()).await;
        assert_eq!(server.request_count(), 0);
    }
    #[tokio::test]
    async fn test_disabled_toggle_row_cannot_suppress_enabled_row() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let mut muted = site_row();
        muted.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        muted.notifications.notify_site_down = false;
        let mut alerting = site_row();
        alerting.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
        notifier
            .site_down(&[&muted, &alerting], &down_result())
            .await;
        assert_eq!(server.request_count(), 1);
    }

    #[tokio::test]
    async fn test_dedup_is_per_channel_destination() {
        let server = TestHttpServer::start().await;
        let notifier = Notifier::new(Client::new(), true);
        let shared_slack_url = format!("{}/slack", server.base_url());
        let mut first = site_row();
        first.notifications.slack_webhook_url = Some(shared_slack_url.clone());
        first.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams-one", server.base_url()));
        let mut second = site_row();
        second.notifications.slack_webhook_url = Some(shared_slack_url);
        second.notifications.microsoft_teams_webhook_url =
            Some(format!("{}/teams-two", server.base_url()));
        notifier.site_down(&[&first, &second], &down_result()).await;
        let one_slack_and_two_teams_sends = 3;
        assert_eq!(server.request_count(), one_slack_and_two_teams_sends);
    }
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_broken_smtp_config_cannot_suppress_working_one() {
        let dead_port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let notifier = Notifier::new(Client::new(), true);
        let mut auth_without_credentials = site_row();
        auth_without_credentials.notifications.smtp.smtp_host = Some("127.0.0.1".to_owned());
        auth_without_credentials.notifications.smtp.smtp_port = Some(dead_port);
        auth_without_credentials.notifications.smtp.smtp_tls_mode =
            crate::models::smtp::SmtpTlsMode::None;
        auth_without_credentials.notifications.smtp.smtp_auth = true;
        auth_without_credentials.notifications.smtp.smtp_from_email =
            Some("alerts@waghorn.tech".to_owned());
        auth_without_credentials.notifications.smtp.smtp_to_email =
            Some("john@waghorn.tech".to_owned());
        let mut same_host_without_auth = site_row();
        same_host_without_auth.notifications.smtp = SmtpSettings {
            smtp_auth: false,
            ..auth_without_credentials.notifications.smtp.clone()
        };
        notifier
            .site_down(
                &[&auth_without_credentials, &same_host_without_auth],
                &down_result(),
            )
            .await;
        let broken_config_failed = "username or password is missing";
        let working_config_reached_the_network = "Connection refused";
        assert!(logs_contain(broken_config_failed));
        assert!(logs_contain(working_config_reached_the_network));
    }
}
