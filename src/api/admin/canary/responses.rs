use crate::canary::{CanarySettings, CanaryState};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct CanarySettingsResponse {
    pub enabled: bool,
    pub url: Option<String>,
    pub timeout_secs: u64,
    pub state: CanaryState,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_succeeded_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl From<CanarySettings> for CanarySettingsResponse {
    fn from(settings: CanarySettings) -> Self {
        let state = settings.state();
        Self {
            enabled: settings.config.enabled,
            url: settings.config.url,
            timeout_secs: settings.config.timeout_secs,
            state,
            last_checked_at: settings.last_checked_at,
            last_succeeded_at: settings.last_succeeded_at,
            last_error: settings.last_error,
        }
    }
}
