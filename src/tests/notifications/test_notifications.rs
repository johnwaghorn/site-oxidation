use crate::models::notifications::TeamNotificationConfig;
use crate::models::site::{CertStatus, SiteRow, SiteStatus};
use crate::models::smtp::SmtpSettings;
use crate::notifications::delivery::PendingDelivery;
use crate::notifications::{Notifier, planning};
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use crate::tests::TestHttpServer;
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::Client;
use sqlx::SqlitePool;

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

async fn deliver(pool: &SqlitePool, notifier: &Notifier, deliveries: Vec<PendingDelivery>) {
    let mut transaction = pool.begin().await.unwrap();
    notifier
        .enqueue(&mut transaction, &deliveries)
        .await
        .unwrap();
    transaction.commit().await.unwrap();
    notifier.process_outbox(pool).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_down_posts_text_payload_to_slack_webhook(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    let deliveries = planning::site_down(&[&site], &down_result()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /slack"));
    assert!(request.contains("Waghorn Technology Ltd"));
    assert!(request.contains("is DOWN"));
    assert!(request.contains("connection refused"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_down_posts_adaptive_card_to_teams_webhook(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.microsoft_teams_webhook_url = Some(format!("{}/teams", server.base_url()));
    let deliveries = planning::site_down(&[&site], &down_result()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /teams"));
    assert!(request.contains("application/vnd.microsoft.card.adaptive"));
    assert!(request.contains("Waghorn Technology Ltd"));
    assert!(request.contains("is DOWN"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_down_respects_notify_flag(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    site.notifications.microsoft_teams_webhook_url = Some(format!("{}/teams", server.base_url()));
    site.notifications.notify_site_down = false;
    let deliveries = planning::site_down(&[&site], &down_result()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_site_recovered_respects_notify_flag(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    site.notifications.microsoft_teams_webhook_url = Some(format!("{}/teams", server.base_url()));
    site.notifications.notify_site_recovered = false;
    let deliveries = planning::site_recovered(&[&site]).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cert_expiring_posts_to_slack_webhook(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    let deliveries = planning::cert_expiring(&[&site], &expiring_cert()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 1);
    let request = server.last_request().unwrap();
    assert!(request.contains("POST /slack"));
    assert!(request.contains("TLS certificate"));
    assert!(request.contains("Waghorn Technology Ltd"));
}

#[sqlx::test(migrations = "./migrations")]
async fn test_cert_expiring_respects_notify_flag(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut site = site_row();
    site.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    site.notifications.microsoft_teams_webhook_url = Some(format!("{}/teams", server.base_url()));
    site.notifications.notify_cert_expiring = false;
    let deliveries = planning::cert_expiring(&[&site], &expiring_cert()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_disabled_toggle_row_cannot_suppress_enabled_row(pool: SqlitePool) {
    let server = TestHttpServer::start().await;
    let notifier = Notifier::new(Client::new(), true);
    let mut muted = site_row();
    muted.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    muted.notifications.notify_site_down = false;
    let mut alerting = site_row();
    alerting.notifications.slack_webhook_url = Some(format!("{}/slack", server.base_url()));
    let deliveries = planning::site_down(&[&muted, &alerting], &down_result()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert_eq!(server.request_count(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_dedup_is_per_channel_destination(pool: SqlitePool) {
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
    let deliveries = planning::site_down(&[&first, &second], &down_result()).unwrap();
    deliver(&pool, &notifier, deliveries).await;
    let one_slack_and_two_teams_sends = 3;
    assert_eq!(server.request_count(), one_slack_and_two_teams_sends);
}

#[sqlx::test(migrations = "./migrations")]
#[tracing_test::traced_test]
async fn test_broken_smtp_config_cannot_suppress_working_one(pool: SqlitePool) {
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
    let deliveries = planning::site_down(
        &[&auth_without_credentials, &same_host_without_auth],
        &down_result(),
    )
    .unwrap();
    deliver(&pool, &notifier, deliveries).await;
    assert!(logs_contain("username or password is missing"));
    assert!(logs_contain("Connection refused"));
}
