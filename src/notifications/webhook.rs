use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

pub(super) async fn post<T: Serialize>(
    client: &Client,
    webhook_url: &str,
    channel: &str,
    payload: &T,
) {
    match client
        .post(webhook_url)
        .json(payload)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {}
        Ok(response) => {
            tracing::warn!(
                "{channel} notification webhook returned status {}",
                response.status()
            );
        }
        Err(error) => {
            tracing::warn!("Failed to send {channel} notification: {error}");
        }
    }
}
