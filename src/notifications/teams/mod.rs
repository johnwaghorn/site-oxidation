mod payloads;

use crate::models::site::SiteRow;
use crate::notifications::webhook;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use reqwest::Client;

pub async fn site_down(client: &Client, webhook_url: &str, site: &SiteRow, result: &ProbeResult) {
    webhook::post(
        client,
        webhook_url,
        "Teams",
        &payloads::site_down(site, result),
    )
    .await;
}

pub async fn site_recovered(client: &Client, webhook_url: &str, site: &SiteRow) {
    webhook::post(
        client,
        webhook_url,
        "Teams",
        &payloads::site_recovered(site),
    )
    .await;
}

pub async fn cert_expiring(client: &Client, webhook_url: &str, site: &SiteRow, cert: &CertCheck) {
    webhook::post(
        client,
        webhook_url,
        "Teams",
        &payloads::cert_expiring(site, cert),
    )
    .await;
}
