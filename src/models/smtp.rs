use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[schema(rename_all = "lowercase", example = "starttls")]
pub enum SmtpTlsMode {
    None,
    #[default]
    StartTls,
    Tls,
}

impl<'de> Deserialize<'de> for SmtpTlsMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        match String::deserialize(deserializer)?.trim() {
            "" => Err(D::Error::custom("SMTP TLS mode is required")),
            "none" => Ok(Self::None),
            "starttls" => Ok(Self::StartTls),
            "tls" => Ok(Self::Tls),
            _ => Err(D::Error::custom("SMTP TLS mode must be a recognised value")),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::FromRow)]
#[allow(clippy::struct_field_names)]
pub struct SmtpSettings {
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_tls_mode: SmtpTlsMode,
    pub smtp_auth: bool,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: Option<String>,
    pub smtp_to_email: Option<String>,
}

impl SmtpSettings {
    pub fn has_delivery_addresses(&self) -> bool {
        self.smtp_host.is_some() && self.smtp_from_email.is_some() && self.smtp_to_email.is_some()
    }
}

impl Default for SmtpSettings {
    fn default() -> Self {
        Self {
            smtp_host: None,
            smtp_port: None,
            smtp_tls_mode: SmtpTlsMode::default(),
            smtp_auth: true,
            smtp_username: None,
            smtp_password: None,
            smtp_from_email: None,
            smtp_to_email: None,
        }
    }
}
