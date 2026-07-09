use serde::Deserialize;
use utoipa::ToSchema;

use super::validators::{
    EmailAddress, SmtpHost, SmtpPassword, SmtpPort, SmtpUsername, TelegramBotToken, TelegramChatId,
    WebhookUrl,
};
use crate::models::smtp::SmtpTlsMode;

#[derive(Deserialize, ToSchema)]
pub struct UpdateTeamNotificationsRequest {
    pub slack_webhook_url: Option<WebhookUrl>,
    pub microsoft_teams_webhook_url: Option<WebhookUrl>,
    pub telegram_bot_token: Option<TelegramBotToken>,
    pub telegram_chat_id: Option<TelegramChatId>,
    pub smtp_host: Option<SmtpHost>,
    pub smtp_port: Option<SmtpPort>,
    pub smtp_tls_mode: Option<SmtpTlsMode>,
    pub smtp_auth: Option<bool>,
    pub smtp_username: Option<SmtpUsername>,
    pub smtp_password: Option<SmtpPassword>,
    pub smtp_from_email: Option<EmailAddress>,
    pub smtp_to_email: Option<EmailAddress>,
    pub notify_site_down: Option<bool>,
    pub notify_site_recovered: Option<bool>,
    pub notify_cert_expiring: Option<bool>,
}

pub struct PatchValue<T> {
    pub provided: bool,
    pub value: Option<T>,
}

impl<T> PatchValue<T> {
    fn omitted() -> Self {
        Self {
            provided: false,
            value: None,
        }
    }

    fn provided(value: Option<T>) -> Self {
        Self {
            provided: true,
            value,
        }
    }
}

pub struct PreparedNotificationUpdate {
    pub slack_webhook_url: PatchValue<String>,
    pub microsoft_teams_webhook_url: PatchValue<String>,
    pub telegram_bot_token: PatchValue<String>,
    pub telegram_chat_id: PatchValue<String>,
    pub smtp_host: PatchValue<String>,
    pub smtp_port: PatchValue<u16>,
    pub smtp_tls_mode: PatchValue<SmtpTlsMode>,
    pub smtp_auth: PatchValue<bool>,
    pub smtp_username: PatchValue<String>,
    pub smtp_password: PatchValue<String>,
    pub smtp_from_email: PatchValue<String>,
    pub smtp_to_email: PatchValue<String>,
    pub notify_site_down: PatchValue<bool>,
    pub notify_site_recovered: PatchValue<bool>,
    pub notify_cert_expiring: PatchValue<bool>,
}

impl PreparedNotificationUpdate {
    pub fn touches_smtp(&self) -> bool {
        self.smtp_host.provided
            || self.smtp_port.provided
            || self.smtp_tls_mode.provided
            || self.smtp_auth.provided
            || self.smtp_username.provided
            || self.smtp_password.provided
            || self.smtp_from_email.provided
            || self.smtp_to_email.provided
    }
}

impl UpdateTeamNotificationsRequest {
    pub fn prepare(self) -> Result<PreparedNotificationUpdate, &'static str> {
        let update = PreparedNotificationUpdate {
            slack_webhook_url: patch(self.slack_webhook_url, WebhookUrl::into_option),
            microsoft_teams_webhook_url: patch(
                self.microsoft_teams_webhook_url,
                WebhookUrl::into_option,
            ),
            telegram_bot_token: patch(self.telegram_bot_token, TelegramBotToken::into_option),
            telegram_chat_id: patch(self.telegram_chat_id, TelegramChatId::into_option),
            smtp_host: patch(self.smtp_host, SmtpHost::into_option),
            smtp_port: patch(self.smtp_port, |value| Some(value.as_u16())),
            smtp_tls_mode: patch(self.smtp_tls_mode, Some),
            smtp_auth: patch(self.smtp_auth, Some),
            smtp_username: patch(self.smtp_username, SmtpUsername::into_option),
            smtp_password: patch(self.smtp_password, SmtpPassword::into_option),
            smtp_from_email: patch(self.smtp_from_email, EmailAddress::into_option),
            smtp_to_email: patch(self.smtp_to_email, EmailAddress::into_option),
            notify_site_down: patch(self.notify_site_down, Some),
            notify_site_recovered: patch(self.notify_site_recovered, Some),
            notify_cert_expiring: patch(self.notify_cert_expiring, Some),
        };
        if update.telegram_bot_token.provided != update.telegram_chat_id.provided {
            return Err("Telegram bot token and chat ID must be updated together");
        }
        if update.telegram_bot_token.provided
            && update.telegram_bot_token.value.is_some() != update.telegram_chat_id.value.is_some()
        {
            return Err("Telegram bot token and chat ID must both be set or both be blank");
        }
        Ok(update)
    }
}

fn patch<T, U>(value: Option<T>, into_value: impl FnOnce(T) -> Option<U>) -> PatchValue<U> {
    value.map_or_else(PatchValue::omitted, |value| {
        PatchValue::provided(into_value(value))
    })
}
