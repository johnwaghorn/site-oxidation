use super::delivery::PendingDelivery;
use super::providers::{email, slack, teams};
use super::{outbox, sender};
use crate::models::smtp::SmtpSettings;
use reqwest::Client;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Notifier {
    client: Client,
    allow_private_smtp_hosts: bool,
    outbox_lock: Arc<Mutex<()>>,
}

impl Notifier {
    pub fn new(client: Client, allow_private_smtp_hosts: bool) -> Self {
        Self {
            client,
            allow_private_smtp_hosts,
            outbox_lock: Arc::new(Mutex::new(())),
        }
    }

    pub(crate) async fn enqueue(
        &self,
        transaction: &mut Transaction<'_, Sqlite>,
        deliveries: &[PendingDelivery],
    ) -> anyhow::Result<()> {
        outbox::enqueue(transaction, deliveries).await
    }

    pub async fn process_outbox(&self, pool: &SqlitePool) {
        let _guard = self.outbox_lock.lock().await;
        if let Err(error) = outbox::process(pool, &self.client, self.allow_private_smtp_hosts).await
        {
            tracing::error!("Failed to process notification outbox: {error}");
        }
    }

    pub async fn test_email(&self, smtp: &SmtpSettings, triggered_by: &str) -> Result<(), String> {
        self.send_test(email::test_delivery(smtp, triggered_by))
            .await
    }

    pub async fn test_slack(&self, webhook_url: &str, triggered_by: &str) -> Result<(), String> {
        let delivery = slack::test_delivery(webhook_url, triggered_by)
            .map_err(|error| format!("could not build the test payload: {error}"))?;
        self.send_test(delivery).await
    }

    pub async fn test_teams(&self, webhook_url: &str, triggered_by: &str) -> Result<(), String> {
        let delivery = teams::test_delivery(webhook_url, triggered_by)
            .map_err(|error| format!("could not build the test payload: {error}"))?;
        self.send_test(delivery).await
    }

    async fn send_test(&self, delivery: PendingDelivery) -> Result<(), String> {
        sender::send(&delivery, &self.client, self.allow_private_smtp_hosts).await
    }
}
