use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

const SEND_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn post<T: Serialize>(
    client: &Client,
    webhook_url: &str,
    payload: &T,
) -> Result<(), String> {
    match client
        .post(webhook_url)
        .json(payload)
        .timeout(SEND_TIMEOUT)
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => Ok(()),
        Ok(response) => Err(format!("webhook returned status {}", response.status())),
        Err(error) => Err(format!("request failed: {error}")),
    }
}
