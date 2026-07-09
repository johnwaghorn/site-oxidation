use crate::models::smtp::SmtpSettings;

#[derive(sqlx::FromRow)]
#[allow(clippy::struct_excessive_bools)]
pub struct TeamNotificationConfig {
    pub slack_webhook_url: Option<String>,
    pub microsoft_teams_webhook_url: Option<String>,
    #[sqlx(flatten)]
    pub smtp: SmtpSettings,
    pub notify_site_down: bool,
    pub notify_site_recovered: bool,
    pub notify_cert_expiring: bool,
}

impl Default for TeamNotificationConfig {
    fn default() -> Self {
        Self {
            slack_webhook_url: None,
            microsoft_teams_webhook_url: None,
            smtp: SmtpSettings::default(),
            notify_site_down: true,
            notify_site_recovered: true,
            notify_cert_expiring: true,
        }
    }
}
