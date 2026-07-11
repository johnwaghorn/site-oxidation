use super::fields::SiteUrl;
use crate::api::errors::ApiErrorResponse;

pub fn ensure_site_url_allowed(
    url: &SiteUrl,
    allow_private_ips: bool,
) -> Result<(), ApiErrorResponse> {
    if !allow_private_ips && url.has_private_ip() {
        return Err(ApiErrorResponse::validation(
            "Private/internal IP addresses are not allowed",
        ));
    }
    Ok(())
}
