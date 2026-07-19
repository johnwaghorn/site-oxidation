use super::delivery::{PendingDelivery, Transport};
use super::{smtp, webhook};
use reqwest::Client;

pub(super) async fn send(
    delivery: &PendingDelivery,
    client: &Client,
    allow_private_smtp_hosts: bool,
) -> Result<(), String> {
    match &delivery.transport {
        Transport::Webhook { url, payload } => webhook::post(client, url, payload).await,
        Transport::Smtp {
            smtp: settings,
            subject,
            body,
        } => smtp::send(settings, subject, body.clone(), allow_private_smtp_hosts).await,
    }
}
