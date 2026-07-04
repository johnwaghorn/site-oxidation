use serde::de::Error;
use serde::{Deserialize, Deserializer};
use url::Url;
use utoipa::ToSchema;

use crate::api::text;

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "https://hooks.slack.com/services/example")]
pub struct WebhookUrl(Option<String>);

impl WebhookUrl {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for WebhookUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let Some(value) = text::optional(&String::deserialize(deserializer)?, "webhook URL", 2048)
            .map_err(D::Error::custom)?
        else {
            return Ok(Self(None));
        };
        let parsed =
            Url::parse(&value).map_err(|_| D::Error::custom("webhook URL must be a valid URL"))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(D::Error::custom("webhook URL must use http or https"));
        }
        Ok(Self(Some(value)))
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "123456:abc")]
pub struct TelegramBotToken(Option<String>);

impl TelegramBotToken {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for TelegramBotToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(
            &String::deserialize(deserializer)?,
            "telegram bot token",
            2048,
        )
        .map(Self)
        .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "123456789")]
pub struct TelegramChatId(Option<String>);

impl TelegramChatId {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for TelegramChatId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(&String::deserialize(deserializer)?, "telegram chat ID", 255)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "smtp.waghorn.tech")]
pub struct SmtpHost(Option<String>);

impl SmtpHost {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for SmtpHost {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(&String::deserialize(deserializer)?, "SMTP host", 255)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "alerts@waghorn.tech")]
pub struct SmtpUsername(Option<String>);

impl SmtpUsername {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for SmtpUsername {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(&String::deserialize(deserializer)?, "SMTP username", 320)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "secret")]
pub struct SmtpPassword(Option<String>);

impl SmtpPassword {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for SmtpPassword {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(&String::deserialize(deserializer)?, "SMTP password", 2048)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "alerts@waghorn.tech")]
pub struct EmailAddress(Option<String>);

impl EmailAddress {
    pub fn into_option(self) -> Option<String> {
        self.0
    }
}

impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        text::optional(&String::deserialize(deserializer)?, "email address", 320)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, ToSchema)]
#[schema(value_type = i64, example = 587)]
pub struct SmtpPort(u16);

impl SmtpPort {
    pub fn as_i64(self) -> i64 {
        i64::from(self.0)
    }
}

impl<'de> Deserialize<'de> for SmtpPort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let port = u16::deserialize(deserializer)?;
        if port == 0 {
            return Err(D::Error::custom("SMTP port must be between 1 and 65535"));
        }
        Ok(Self(port))
    }
}

#[derive(Debug, Clone, Copy, ToSchema)]
#[schema(rename_all = "lowercase", example = "starttls")]
pub enum SmtpSecurity {
    None,
    StartTls,
    Tls,
}

impl SmtpSecurity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::StartTls => "starttls",
            Self::Tls => "tls",
        }
    }
}

impl<'de> Deserialize<'de> for SmtpSecurity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.trim() {
            "" => Err(D::Error::custom("SMTP security is required")),
            "none" => Ok(Self::None),
            "starttls" => Ok(Self::StartTls),
            "tls" => Ok(Self::Tls),
            _ => Err(D::Error::custom("SMTP security must be a recognised value")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("https://hooks.slack.com/services/example", true)]
    #[case(" http://waghorn.tech/webhook ", true)]
    #[case("ftp://waghorn.tech/webhook", false)]
    #[case("not a url", false)]
    #[case("", true)]
    fn test_webhook_url(#[case] value: &str, #[case] valid: bool) {
        let result: Result<WebhookUrl, _> = serde_json::from_value(json!(value));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case(1, true)]
    #[case(587, true)]
    #[case(65535, true)]
    #[case(0, false)]
    #[case(65536, false)]
    fn test_smtp_port(#[case] value: i64, #[case] valid: bool) {
        let result: Result<SmtpPort, _> = serde_json::from_value(json!(value));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case("none", true)]
    #[case("starttls", true)]
    #[case("tls", true)]
    #[case("ssl", false)]
    fn test_smtp_security(#[case] value: &str, #[case] valid: bool) {
        let result: Result<SmtpSecurity, _> = serde_json::from_value(json!(value));
        assert_eq!(result.is_ok(), valid);
    }
}
