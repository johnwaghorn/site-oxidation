use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Serialize, FromRow, ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct TeamNotificationsResponse {
    pub team_id: i64,
    pub slack_webhook_url: Option<String>,
    pub microsoft_teams_webhook_url: Option<String>,
    pub telegram_bot_token_set: bool,
    pub telegram_chat_id: Option<String>,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_security: String,
    pub smtp_auth: bool,
    pub smtp_username: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_to_email: Option<String>,
    pub smtp_password_set: bool,
    pub notify_site_down: bool,
    pub notify_site_recovered: bool,
    pub notify_cert_expiring: bool,
}
