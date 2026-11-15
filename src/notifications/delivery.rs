use crate::models::smtp::SmtpSettings;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub(crate) struct PendingDelivery {
    pub(super) provider: String,
    pub(super) summary: Option<String>,
    pub(super) team_id: Option<i64>,
    pub(super) team: Option<String>,
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
        summary: &str,
        payload: &T,
    ) -> serde_json::Result<Self> {
        Ok(Self {
            provider: provider.to_owned(),
            summary: Some(summary.to_owned()),
            team_id: None,
            team: None,
            transport: Transport::Webhook {
                url: url.to_owned(),
                payload: serde_json::to_value(payload)?,
            },
        })
    }

    pub(super) fn smtp(smtp: SmtpSettings, subject: String, body: String) -> Self {
        Self {
            provider: "Email".to_owned(),
            summary: Some(subject.clone()),
            team_id: None,
            team: None,
            transport: Transport::Smtp {
                smtp,
                subject,
                body,
            },
        }
    }

    pub(crate) fn for_team(mut self, team_id: Option<i64>, team_name: Option<&str>) -> Self {
        self.team_id = team_id;
        self.team = team_name.map(str::to_owned);
        self
    }
}
