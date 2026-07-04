mod payloads;

use crate::models::site::SiteRow;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use payloads::SlackPayload;
use reqwest::Client;
use std::time::Duration;

pub async fn site_down(client: &Client, webhook_url: &str, site: &SiteRow, result: &ProbeResult) {
    send(client, webhook_url, payloads::site_down(site, result)).await;
}

pub async fn site_recovered(client: &Client, webhook_url: &str, site: &SiteRow) {
    send(client, webhook_url, payloads::site_recovered(site)).await;
}

pub async fn cert_expiring(client: &Client, webhook_url: &str, site: &SiteRow, cert: &CertCheck) {
    send(client, webhook_url, payloads::cert_expiring(site, cert)).await;
}

async fn send(client: &Client, webhook_url: &str, payload: SlackPayload) {
    match client
        .post(webhook_url)
        .json(&payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {}
        Ok(response) => {
            tracing::warn!(
                "Slack notification webhook returned status {}",
                response.status()
            );
        }
        Err(error) => {
            tracing::warn!("Failed to send Slack notification: {}", error);
        }
    }
}
