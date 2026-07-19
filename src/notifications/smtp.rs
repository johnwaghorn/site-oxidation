use crate::models::smtp::{SmtpSettings, SmtpTlsMode};
use crate::security::resolver::{ResolveError, resolve_public_addrs};
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::time::Duration;

const SEND_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) async fn send(
    smtp: &SmtpSettings,
    subject: &str,
    body: String,
    allow_private_hosts: bool,
) -> Result<(), String> {
    let (Some(host), Some(from), Some(to)) =
        (&smtp.smtp_host, &smtp.smtp_from_email, &smtp.smtp_to_email)
    else {
        return Err("SMTP is not fully configured".to_owned());
    };
    ensure_host_permitted(host, allow_private_hosts).await?;
    let message = Message::builder()
        .from(parse_mailbox(from)?)
        .to(parse_mailbox(to)?)
        .subject(subject)
        .body(body)
        .map_err(|error| format!("could not build the message: {error}"))?;
    let transport = build_transport(smtp, host)?;
    transport
        .send(message)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

// Even with the check below, a private host can still slip through, letting a
// crafted SMTP config make the server open connections inside its own network
// (SSRF): lettre does its own DNS lookup when connecting, so a DNS record that
// changes between the two lookups is not caught.
async fn ensure_host_permitted(host: &str, allow_private_hosts: bool) -> Result<(), String> {
    if allow_private_hosts {
        return Ok(());
    }
    // Port 0: we only need IP resolution, not a specific port.
    let resolved = tokio::time::timeout(SEND_TIMEOUT, resolve_public_addrs(host, 0))
        .await
        .map_err(|_| format!("timed out resolving SMTP host '{host}'"))?;
    resolved.map(|_| ()).map_err(|error| match error {
        ResolveError::PrivateIp { .. } => smtp_private_host_error(host),
        ResolveError::Lookup(error) => format!("could not resolve SMTP host '{host}': {error}"),
    })
}

fn smtp_private_host_error(host: &str) -> String {
    format!(
        "SMTP host '{host}' resolves to a private address; set SMTP_ALLOW_PRIVATE_HOSTS=true to allow it"
    )
}

fn parse_mailbox(address: &str) -> Result<Mailbox, String> {
    address
        .parse()
        .map_err(|error| format!("invalid email address '{address}': {error}"))
}

fn build_transport(
    smtp: &SmtpSettings,
    host: &str,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let (builder, default_port_for_mode) = match smtp.smtp_tls_mode {
        SmtpTlsMode::Tls => (AsyncSmtpTransport::<Tokio1Executor>::relay(host), 465),
        SmtpTlsMode::StartTls => (
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host),
            587,
        ),
        SmtpTlsMode::None => (
            Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
                host,
            )),
            25,
        ),
    };
    let port = smtp.smtp_port.unwrap_or(default_port_for_mode);
    let mut builder = builder
        .map_err(|error| format!("could not configure SMTP transport for '{host}': {error}"))?
        .timeout(Some(SEND_TIMEOUT))
        .port(port);
    if smtp.smtp_auth {
        let (Some(username), Some(password)) = (&smtp.smtp_username, &smtp.smtp_password) else {
            return Err(
                "SMTP authentication is enabled but the username or password is missing".to_owned(),
            );
        };
        builder = builder.credentials(Credentials::new(username.clone(), password.clone()));
    }
    Ok(builder.build())
}
