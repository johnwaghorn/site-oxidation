use crate::models::smtp::SmtpSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub(crate) struct PendingDelivery {
    pub(super) provider: String,
    pub(super) transport: Transport,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum Transport {
    Webhook {
        url: String,
        payload: Value,
    },
    Smtp {
        smtp: SmtpSettings,
        subject: String,
        body: String,
    },
}

impl PendingDelivery {
    pub(crate) fn webhook<T: Serialize>(
        provider: &str,
        url: &str,
        payload: &T,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            provider: provider.to_owned(),
            transport: Transport::Webhook {
                url: url.to_owned(),
                payload: serde_json::to_value(payload)?,
            },
        })
    }

    pub(super) fn smtp(smtp: SmtpSettings, subject: String, body: String) -> Self {
        Self {
            provider: "Email".to_owned(),
            transport: Transport::Smtp {
                smtp,
                subject,
                body,
            },
        }
    }
}
