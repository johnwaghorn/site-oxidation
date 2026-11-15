use crate::models::site::SiteRow;
use crate::notifications::delivery::PendingDelivery;
use crate::notifications::format;
use crate::notifications::providers::TEST_NOTIFICATION_TITLE;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;

mod payloads;

pub(in crate::notifications) fn test_delivery(
    webhook_url: &str,
    triggered_by: &str,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook(
        "Microsoft Teams",
        webhook_url,
        TEST_NOTIFICATION_TITLE,
        &payloads::test_notification(triggered_by),
    )
}

pub(in crate::notifications) fn site_down_delivery(
    webhook_url: &str,
    site: &SiteRow,
    result: &ProbeResult,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook(
        "Microsoft Teams",
        webhook_url,
        &format::site_down_title(site),
        &payloads::site_down(site, result),
    )
}

pub(in crate::notifications) fn site_recovered_delivery(
    webhook_url: &str,
    site: &SiteRow,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook(
        "Microsoft Teams",
        webhook_url,
        &format::site_recovered_title(site),
        &payloads::site_recovered(site),
    )
}

pub(in crate::notifications) fn cert_expiring_delivery(
    webhook_url: &str,
    site: &SiteRow,
    cert: &CertCheck,
) -> serde_json::Result<PendingDelivery> {
    PendingDelivery::webhook(
        "Microsoft Teams",
        webhook_url,
        &format::cert_expiring_title(site, cert),
        &payloads::cert_expiring(site, cert),
    )
}
