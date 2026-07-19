use super::delivery::PendingDelivery;
use super::providers::{email, slack, teams};
use crate::models::site::SiteRow;
use crate::models::smtp::SmtpSettings;
use crate::probe::cert::CertCheck;
use crate::probe::http::ProbeResult;
use std::collections::HashSet;

#[derive(Default)]
struct SeenTargets<'a> {
    slack_urls: HashSet<&'a str>,
    teams_urls: HashSet<&'a str>,
    smtp_configs: HashSet<&'a SmtpSettings>,
}

impl<'a> SeenTargets<'a> {
    fn should_send_email(&mut self, site: &'a SiteRow) -> bool {
        site.notifications.smtp.has_delivery_addresses()
            && self.smtp_configs.insert(&site.notifications.smtp)
    }
}

pub(crate) fn site_down(
    sites: &[&SiteRow],
    result: &ProbeResult,
) -> anyhow::Result<Vec<PendingDelivery>> {
    let mut deliveries = Vec::new();
    let mut targets = SeenTargets::default();
    for &site in sites {
        if !site.notifications.notify_site_down {
            continue;
        }
        if let Some(url) = site.notifications.slack_webhook_url.as_deref()
            && targets.slack_urls.insert(url)
        {
            deliveries.push(slack::site_down_delivery(url, site, result)?);
        }
        if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
            && targets.teams_urls.insert(url)
        {
            deliveries.push(teams::site_down_delivery(url, site, result)?);
        }
        if targets.should_send_email(site) {
            deliveries.push(email::site_down_delivery(
                &site.notifications.smtp,
                site,
                result,
            ));
        }
    }
    Ok(deliveries)
}

pub(crate) fn site_recovered(sites: &[&SiteRow]) -> anyhow::Result<Vec<PendingDelivery>> {
    let mut deliveries = Vec::new();
    let mut targets = SeenTargets::default();
    for &site in sites {
        if !site.notifications.notify_site_recovered {
            continue;
        }
        if let Some(url) = site.notifications.slack_webhook_url.as_deref()
            && targets.slack_urls.insert(url)
        {
            deliveries.push(slack::site_recovered_delivery(url, site)?);
        }
        if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
            && targets.teams_urls.insert(url)
        {
            deliveries.push(teams::site_recovered_delivery(url, site)?);
        }
        if targets.should_send_email(site) {
            deliveries.push(email::site_recovered_delivery(
                &site.notifications.smtp,
                site,
            ));
        }
    }
    Ok(deliveries)
}

pub(crate) fn cert_expiring(
    sites: &[&SiteRow],
    cert: &CertCheck,
) -> anyhow::Result<Vec<PendingDelivery>> {
    let mut deliveries = Vec::new();
    let mut targets = SeenTargets::default();
    for &site in sites {
        if !site.notifications.notify_cert_expiring {
            continue;
        }
        if let Some(url) = site.notifications.slack_webhook_url.as_deref()
            && targets.slack_urls.insert(url)
        {
            deliveries.push(slack::cert_expiring_delivery(url, site, cert)?);
        }
        if let Some(url) = site.notifications.microsoft_teams_webhook_url.as_deref()
            && targets.teams_urls.insert(url)
        {
            deliveries.push(teams::cert_expiring_delivery(url, site, cert)?);
        }
        if targets.should_send_email(site) {
            deliveries.push(email::cert_expiring_delivery(
                &site.notifications.smtp,
                site,
                cert,
            ));
        }
    }
    Ok(deliveries)
}
