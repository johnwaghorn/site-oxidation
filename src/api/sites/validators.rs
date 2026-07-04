use crate::api::text;
use crate::security::ip::is_private_ip;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::net::IpAddr;
use url::{Host, Url};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "Waghorn Technology Ltd")]
pub struct SiteName(String);

impl SiteName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SiteName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let site_name = text::required(&String::deserialize(deserializer)?, "site name", 100)
            .map_err(D::Error::custom)?;
        Ok(SiteName(site_name))
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "https://waghorn.tech")]
pub struct SiteUrl(String);

impl SiteUrl {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn has_private_ip(&self) -> bool {
        let url = Url::parse(&self.0).ok();
        match url.as_ref().and_then(Url::host) {
            Some(Host::Ipv4(ip)) => is_private_ip(&IpAddr::V4(ip)),
            Some(Host::Ipv6(ip)) => is_private_ip(&IpAddr::V6(ip)),
            _ => false,
        }
    }
}

impl<'de> Deserialize<'de> for SiteUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = text::required(&String::deserialize(deserializer)?, "site url", 2000)
            .map_err(D::Error::custom)?;
        if raw.chars().count() < 10 {
            return Err(D::Error::custom("site url is too short"));
        }
        let parsed =
            Url::parse(&raw).map_err(|_| D::Error::custom("site url must be a valid URL"))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(D::Error::custom(
                "site url must start with https:// or http://",
            ));
        }
        Ok(SiteUrl(raw))
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = i64, example = 200)]
pub struct ExpectedStatus(i64);

impl ExpectedStatus {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for ExpectedStatus {
    fn default() -> Self {
        Self(200)
    }
}

impl<'de> Deserialize<'de> for ExpectedStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status = i64::deserialize(deserializer)?;
        if !(100..=599).contains(&status) {
            return Err(D::Error::custom("status code must be between 100 and 599"));
        }
        Ok(ExpectedStatus(status))
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "Waghorn Technology")]
pub struct ExpectedText(String);

impl ExpectedText {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ExpectedText {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let expected_text = String::deserialize(deserializer)?;
        if expected_text.is_empty() {
            return Err(D::Error::custom("text is too short"));
        }
        text::check_max_chars(&expected_text, "text", 500).map_err(D::Error::custom)?;
        Ok(ExpectedText(expected_text))
    }
}

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = i64, example = 60)]
pub struct CheckInterval(i64);
impl CheckInterval {
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl Default for CheckInterval {
    fn default() -> Self {
        Self(60)
    }
}

impl<'de> Deserialize<'de> for CheckInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let check_interval = i64::deserialize(deserializer)?;
        if !(60..=3600).contains(&check_interval) {
            return Err(D::Error::custom(
                "check interval must be between 60 and 3600",
            ));
        }
        Ok(CheckInterval(check_interval))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("Waghorn Technology Ltd", true)]
    #[case("A", true)]
    #[case(&"a".repeat(100), true)]
    #[case("", false)]
    #[case(&"a".repeat(101), false)]
    fn test_site_name(#[case] name: &str, #[case] valid: bool) {
        let result: Result<SiteName, _> = serde_json::from_value(json!(name));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case("https://waghorn.tech", true)]
    #[case("waghorn.tech", false)]
    #[case("ftp://waghorn.tech", false)]
    #[case("http", false)]
    #[case(&"a".repeat(2001), false)]
    #[case("http://[::1", false)]
    fn test_site_url(#[case] url: &str, #[case] valid: bool) {
        let result: Result<SiteUrl, _> = serde_json::from_value(json!(url));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case(200, true)]
    #[case(100, true)]
    #[case(599, true)]
    #[case(99, false)]
    #[case(600, false)]
    #[case(0, false)]
    fn test_expected_status(#[case] status: i64, #[case] valid: bool) {
        let result: Result<ExpectedStatus, _> = serde_json::from_value(json!(status));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case("Waghorn Technology", true)]
    #[case("a", true)]
    #[case(&"a".repeat(500), true)]
    #[case("", false)]
    #[case(&"a".repeat(501), false)]
    fn test_expected_text(#[case] text: &str, #[case] valid: bool) {
        let result: Result<ExpectedText, _> = serde_json::from_value(json!(text));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case(60, true)]
    #[case(300, true)]
    #[case(3600, true)]
    #[case(59, false)]
    #[case(3601, false)]
    #[case(0, false)]
    fn test_check_interval(#[case] interval: i64, #[case] valid: bool) {
        let result: Result<CheckInterval, _> = serde_json::from_value(json!(interval));
        assert_eq!(result.is_ok(), valid);
    }

    #[rstest]
    #[case("http://192.168.1.1/admin", true)]
    #[case("http://127.0.0.1:8080", true)]
    #[case("http://10.0.0.1/path", true)]
    #[case("http://172.16.0.1:3000", true)]
    #[case("http://169.254.1.1", true)]
    #[case("http://8.8.8.8/dns", false)]
    #[case("https://waghorn.tech", false)]
    #[case("http://localhost:3000", false)]
    fn test_has_private_ip(#[case] url: &str, #[case] expected: bool) {
        let site_url: SiteUrl = serde_json::from_value(json!(url)).unwrap();
        assert_eq!(site_url.has_private_ip(), expected);
    }
}
