use super::responses::TeamNotificationsResponse;

pub fn merged_smtp_config_error(merged: &TeamNotificationsResponse) -> Option<&'static str> {
    let delivery_addresses = [
        &merged.smtp_host,
        &merged.smtp_from_email,
        &merged.smtp_to_email,
    ];
    let set_count = delivery_addresses
        .iter()
        .filter(|field| field.is_some())
        .count();
    if set_count != 0 && set_count != delivery_addresses.len() {
        return Some("SMTP host, from address and to address must be saved together");
    }
    let email_enabled = set_count == delivery_addresses.len();
    if email_enabled
        && merged.smtp_auth
        && (merged.smtp_username.is_none() || !merged.smtp_password_set)
    {
        return Some("SMTP authentication is enabled but the username or password is missing");
    }
    None
}
