mod payloads;

use crate::models::site::SiteRow;
use crate::notifications::delivery::PendingDelivery;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;

pub(in crate::notifications) fn test_delivery(
    webhook_url: &str,
    triggered_by: &str,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook("Slack", webhook_url, &payloads::test(triggered_by))
}

pub(in crate::notifications) fn site_down_delivery(
    webhook_url: &str,
    site: &SiteRow,
    result: &ProbeResult,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook("Slack", webhook_url, &payloads::site_down(site, result))
}

pub(in crate::notifications) fn site_recovered_delivery(
    webhook_url: &str,
    site: &SiteRow,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook("Slack", webhook_url, &payloads::site_recovered(site))
}

pub(in crate::notifications) fn cert_expiring_delivery(
    webhook_url: &str,
    site: &SiteRow,
    cert: &CertCheck,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook("Slack", webhook_url, &payloads::cert_expiring(site, cert))
}
