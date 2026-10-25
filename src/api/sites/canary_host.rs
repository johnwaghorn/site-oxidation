use super::responses::SiteResponse;
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

pub(crate) async fn flag_sites_sharing_canary_host(
    pool: &SqlitePool,
    sites: &mut [SiteResponse],
) -> Result<(), ApiErrorResponse> {
    let settings = canary::load_settings(pool)
        .await
        .map_err(|error| internal_err("Failed to load canary settings", error))?;
    if !settings.config.enabled {
        return Ok(());
    }
    let Some(canary_host) = settings.config.url.as_deref().and_then(normalised_url_host) else {
        return Ok(());
    };
    for site in sites {
        site.shares_host_with_canary = site_url_matches_canary_host(&site.url, &canary_host);
    }
    Ok(())
}
