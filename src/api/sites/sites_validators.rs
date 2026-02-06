use serde::de::Error;
use serde::{Deserialize, Deserializer};
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
        let site_name = String::deserialize(deserializer)?;
        let len = site_name.chars().count();
        if len < 1 {
            return Err(D::Error::custom("site name is too short"));
        }
        if len > 100 {
            return Err(D::Error::custom("site name is too long"));
        }
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
}

impl<'de> Deserialize<'de> for SiteUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let site_url = String::deserialize(deserializer)?;
        let len = site_url.chars().count();
        if len < 10 {
            return Err(D::Error::custom("site url is too short"));
        }
        if len > 2000 {
            return Err(D::Error::custom("site url is too long"));
        }
        if !site_url.starts_with("http://") && !site_url.starts_with("https://") {
            return Err(D::Error::custom(
                "site url must start with https:// or http://",
            ));
        }
        Ok(SiteUrl(site_url))
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
        let text = String::deserialize(deserializer)?;
        let len = text.chars().count();
        if len < 1 {
            return Err(D::Error::custom("text is too short"));
        }
        if len > 500 {
            return Err(D::Error::custom("text is too long"));
        }
        Ok(ExpectedText(text))
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
    use serde_json::json;
    #[test]
    fn test_site_name() {
        let site_name: SiteName = serde_json::from_value(json!("Waghorn Technology Ltd")).unwrap();
        assert_eq!(site_name.as_str(), "Waghorn Technology Ltd");
    }
    #[test]
    fn test_site_name_too_long_rejected() {
        let long_name = "a".repeat(101);
        let result: Result<SiteName, _> = serde_json::from_value(json!(long_name));
        assert!(result.is_err());
    }

    #[test]
    fn test_site_name_empty_rejected() {
        let result: Result<SiteName, _> = serde_json::from_value(json!(""));
        assert!(result.is_err());
    }

    #[test]
    fn test_site_url() {
        let url: SiteUrl = serde_json::from_value(json!("https://waghorn.tech")).unwrap();
        assert_eq!(url.as_str(), "https://waghorn.tech");
    }

    #[test]
    fn test_site_url_missing_protocol_rejected() {
        let result: Result<SiteUrl, _> = serde_json::from_value(json!("waghorn.tech"));
        assert!(result.is_err());
    }

    #[test]
    fn test_expected_status() {
        let status: ExpectedStatus = serde_json::from_value(json!(200)).unwrap();
        assert_eq!(status.as_i64(), 200);
    }

    #[test]
    fn test_expected_status_below_100_rejected() {
        let result: Result<ExpectedStatus, _> = serde_json::from_value(json!(99));
        assert!(result.is_err());
    }

    #[test]
    fn test_expected_status_above_599_rejected() {
        let result: Result<ExpectedStatus, _> = serde_json::from_value(json!(600));
        assert!(result.is_err());
    }

    #[test]
    fn test_expected_text_valid() {
        let text: ExpectedText = serde_json::from_value(json!("Waghorn Technology")).unwrap();
        assert_eq!(text.as_str(), "Waghorn Technology");
    }

    #[test]
    fn test_expected_text_too_long_rejected() {
        let long_text = "a".repeat(501);
        let result: Result<ExpectedText, _> = serde_json::from_value(json!(long_text));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_interval() {
        let check_interval: CheckInterval = serde_json::from_value(json!(60)).unwrap();
        assert_eq!(check_interval.as_i64(), 60);
    }

    #[test]
    fn test_check_interval_below_60_rejected() {
        let result: Result<CheckInterval, _> = serde_json::from_value(json!(59));
        assert!(result.is_err());
    }

    #[test]
    fn test_check_interval_above_3600_rejected() {
        let result: Result<CheckInterval, _> = serde_json::from_value(json!(3601));
        assert!(result.is_err());
    }
}
