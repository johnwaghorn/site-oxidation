use crate::api::errors::ApiErrorResponse;
use crate::canary::CanaryConfig;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UpdateCanarySettingsRequest {
    pub enabled: bool,
    pub url: Option<String>,
    pub timeout_secs: u64,
}

impl UpdateCanarySettingsRequest {
    pub fn into_canary_config(self) -> Result<CanaryConfig, ApiErrorResponse> {
        let url = self
            .url
            .map(|url| url.trim().to_owned())
            .filter(|url| !url.is_empty());
        if self.enabled && url.is_none() {
            return Err(ApiErrorResponse::validation(
                "Canary URL is required when the canary is enabled",
            ));
        }
        if let Some(url) = &url {
            if !(10..=2048).contains(&url.len()) {
                return Err(ApiErrorResponse::validation(
                    "Canary URL must be between 10 and 2048 characters",
                ));
            }
            let parsed = reqwest::Url::parse(url)
                .map_err(|_| ApiErrorResponse::validation("Canary URL must be a valid URL"))?;
            if !matches!(parsed.scheme(), "http" | "https") || parsed.host().is_none() {
                return Err(ApiErrorResponse::validation(
                    "Canary URL must use HTTP or HTTPS and include a host",
                ));
            }
        }
        if !(1..=300).contains(&self.timeout_secs) {
            return Err(ApiErrorResponse::validation(
                "Canary timeout must be between 1 and 300 seconds",
            ));
        }
        Ok(CanaryConfig {
            enabled: self.enabled,
            url,
            timeout_secs: self.timeout_secs,
        })
    }
}
