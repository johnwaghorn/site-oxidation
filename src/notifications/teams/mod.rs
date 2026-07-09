mod payloads;

use crate::models::site::SiteRow;
use crate::notifications::webhook;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use reqwest::Client;

pub async fn test(client: &Client, webhook_url: &str, triggered_by: &str) -> Result<(), String> {
    webhook::post(client, webhook_url, &payloads::test(triggered_by)).await
}

pub async fn site_down(client: &Client, webhook_url: &str, site: &SiteRow, result: &ProbeResult) {
    let payload = payloads::site_down(site, result);
    if let Err(error) = webhook::post(client, webhook_url, &payload).await {
        tracing::warn!("Microsoft Teams site-down notification failed: {error}");
    }
}

pub async fn site_recovered(client: &Client, webhook_url: &str, site: &SiteRow) {
    let payload = payloads::site_recovered(site);
    if let Err(error) = webhook::post(client, webhook_url, &payload).await {
        tracing::warn!("Microsoft Teams site-recovered notification failed: {error}");
    }
}

pub async fn cert_expiring(client: &Client, webhook_url: &str, site: &SiteRow, cert: &CertCheck) {
    let payload = payloads::cert_expiring(site, cert);
    if let Err(error) = webhook::post(client, webhook_url, &payload).await {
        tracing::warn!("Microsoft Teams cert-expiring notification failed: {error}");
    }
}
