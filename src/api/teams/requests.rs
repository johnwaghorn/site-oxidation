use serde::Deserialize;
use utoipa::ToSchema;

use super::fields::{EmailAddress, SmtpHost, SmtpPassword, SmtpPort, SmtpUsername, WebhookUrl};
use crate::models::smtp::SmtpTlsMode;

#[derive(Deserialize, ToSchema)]
pub struct UpdateTeamNotificationsRequest {
    pub slack_webhook_url: Option<WebhookUrl>,
    pub microsoft_teams_webhook_url: Option<WebhookUrl>,
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

    fn set_or_cleared(&self) -> Option<&'static str> {
        self.provided.then_some(if self.value.is_some() {
            "set"
        } else {
            "cleared"
        })
    }
}

impl PatchValue<bool> {
    fn on_or_off(&self) -> Option<&'static str> {
        self.provided.then_some(if self.value == Some(true) {
            "on"
        } else {
            "off"
        })
    }
}

pub struct PreparedNotificationUpdate {
    pub slack_webhook_url: PatchValue<String>,
    pub microsoft_teams_webhook_url: PatchValue<String>,
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
    pub fn describe_settings_in_request_without_secrets(&self) -> Vec<String> {
        [
            ("slack_webhook_url", self.slack_webhook_url.set_or_cleared()),
            (
                "microsoft_teams_webhook_url",
                self.microsoft_teams_webhook_url.set_or_cleared(),
            ),
            ("smtp_host", self.smtp_host.set_or_cleared()),
            ("smtp_port", self.smtp_port.set_or_cleared()),
            ("smtp_tls_mode", self.smtp_tls_mode.set_or_cleared()),
            ("smtp_auth", self.smtp_auth.on_or_off()),
            ("smtp_username", self.smtp_username.set_or_cleared()),
            ("smtp_password", self.smtp_password.set_or_cleared()),
            ("smtp_from_email", self.smtp_from_email.set_or_cleared()),
            ("smtp_to_email", self.smtp_to_email.set_or_cleared()),
            ("notify_site_down", self.notify_site_down.on_or_off()),
            (
                "notify_site_recovered",
                self.notify_site_recovered.on_or_off(),
            ),
            (
                "notify_cert_expiring",
                self.notify_cert_expiring.on_or_off(),
            ),
        ]
        .into_iter()
        .filter_map(|(setting_name, what_happened)| {
            what_happened.map(|what_happened| format!("{setting_name} {what_happened}"))
        })
        .collect()
    }

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
    pub fn prepare(self) -> PreparedNotificationUpdate {
        PreparedNotificationUpdate {
            slack_webhook_url: patch(self.slack_webhook_url, WebhookUrl::into_option),
            microsoft_teams_webhook_url: patch(
                self.microsoft_teams_webhook_url,
                WebhookUrl::into_option,
            ),
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
        }
    }
}

fn patch<T, U>(value: Option<T>, into_value: impl FnOnce(T) -> Option<U>) -> PatchValue<U> {
    value.map_or_else(PatchValue::omitted, |value| {
        PatchValue::provided(into_value(value))
    })
}
