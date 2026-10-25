use super::responses::{CanaryWarning, SiteResponse};
use crate::api::errors::{ApiErrorResponse, internal_err};
use crate::canary;
use sqlx::SqlitePool;

fn normalised_url_host(url: &str) -> Option<String> {
    let parsed_url = reqwest::Url::parse(url).ok()?;
    let host = parsed_url.host_str()?.trim_end_matches('.');
    Some(host.strip_prefix("www.").unwrap_or(host).to_owned())
}

fn site_url_matches_canary_host(site_url: &str, canary_host: &str) -> bool {
    normalised_url_host(site_url).is_some_and(|site_host| site_host == canary_host)
}

pub(crate) async fn set_canary_warnings(
    pool: &SqlitePool,
    sites: &mut [SiteResponse],
) -> Result<(), ApiErrorResponse> {
    let settings = canary::load_settings(pool)
        .await
        .map_err(|error| internal_err("Failed to load canary settings", error))?;
    let history = canary::load_history_summary(pool)
        .await
        .map_err(|error| internal_err("Failed to load canary history", error))?;
    let active_canary_url = settings.config.url.filter(|_| settings.config.enabled);
    let Some(canary_host) = active_canary_url.as_deref().and_then(normalised_url_host) else {
        for site in sites {
            site.canary_warning = Some(CanaryWarning::Inactive);
            site.canary_warning_since = history.last_ended_at;
        }
        return Ok(());
    };
    for site in sites {
        if site_url_matches_canary_host(&site.url, &canary_host) {
            site.canary_warning = Some(CanaryWarning::SharesHost);
            site.canary_warning_since = history.active_since;
        }
    }
    Ok(())
}
