mod payloads;

use crate::models::site::SiteRow;
use crate::models::smtp::SmtpSettings;
use crate::notifications::delivery::PendingDelivery;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;

pub(in crate::notifications) fn test_delivery(
    smtp: &SmtpSettings,
    triggered_by: &str,
) -> PendingDelivery {
    PendingDelivery::smtp(
        smtp.clone(),
        payloads::test_subject().to_owned(),
        payloads::test_body(triggered_by),
    )
}

pub(in crate::notifications) fn site_down_delivery(
    smtp: &SmtpSettings,
    site: &SiteRow,
    result: &ProbeResult,
) -> PendingDelivery {
    PendingDelivery::smtp(
        smtp.clone(),
        payloads::site_down_subject(site),
        payloads::site_down_body(site, result),
    )
}

pub(in crate::notifications) fn site_recovered_delivery(
    smtp: &SmtpSettings,
    site: &SiteRow,
) -> PendingDelivery {
    PendingDelivery::smtp(
        smtp.clone(),
        payloads::site_recovered_subject(site),
        payloads::site_recovered_body(site),
    )
}

pub(in crate::notifications) fn cert_expiring_delivery(
    smtp: &SmtpSettings,
    site: &SiteRow,
    cert: &CertCheck,
) -> PendingDelivery {
    PendingDelivery::smtp(
        smtp.clone(),
        payloads::cert_expiring_subject(site, cert),
        payloads::cert_expiring_body(site, cert),
    )
}
