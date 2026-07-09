use crate::models::site::CertStatus;
use crate::security::resolver::{resolve_addrs, resolve_public_addrs, warn_probe_private_host};
use chrono::{DateTime, Utc};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::CryptoProvider;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, SignatureScheme};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector, rustls};
use x509_cert::Certificate;
use x509_cert::der::Decode;

pub struct CertCheck {
    pub status: CertStatus,
    pub expires_at: Option<DateTime<Utc>>,
}

struct CertFacts {
    expires_at: DateTime<Utc>,
    trusted: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct CertExpiryWindows {
    pub warn_days: i64,
    pub critical_days: i64,
}

pub async fn check_certificate(
    url: &str,
    allow_untrusted: bool,
    allow_private_ips: bool,
    timeout: Duration,
    now: DateTime<Utc>,
    windows: CertExpiryWindows,
) -> CertCheck {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return CertCheck {
            status: CertStatus::Invalid,
            expires_at: None,
        };
    };
    if parsed.scheme() != "https" {
        return CertCheck {
            status: CertStatus::None,
            expires_at: None,
        };
    }
    let Some(host) = parsed.host_str() else {
        return CertCheck {
            status: CertStatus::Invalid,
            expires_at: None,
        };
    };
    let port = parsed.port().unwrap_or(443);
    match tokio::time::timeout(
        timeout,
        inspect_certificate(host, port, allow_private_ips, allow_untrusted),
    )
    .await
    {
        Ok(Ok(facts)) => CertCheck {
            status: classify_cert_status(
                Some(facts.expires_at),
                facts.trusted,
                allow_untrusted,
                now,
                windows,
            ),
            expires_at: Some(facts.expires_at),
        },
        Ok(Err(_)) | Err(_) => CertCheck {
            status: CertStatus::Invalid,
            expires_at: None,
        },
    }
}

fn classify_cert_status(
    expires_at: Option<DateTime<Utc>>,
    trusted: bool,
    allow_untrusted: bool,
    now: DateTime<Utc>,
    windows: CertExpiryWindows,
) -> CertStatus {
    let Some(expires_at) = expires_at else {
        return CertStatus::Invalid;
    };
    if expires_at <= now {
        return CertStatus::Expired;
    }
    if !allow_untrusted && !trusted {
        return CertStatus::Invalid;
    }
    let days_left = expires_at.signed_duration_since(now).num_days();
    if days_left <= windows.critical_days {
        CertStatus::Critical
    } else if days_left <= windows.warn_days {
        CertStatus::Expiring
    } else {
        CertStatus::Valid
    }
}

async fn inspect_certificate(
    host: &str,
    port: u16,
    allow_private_ips: bool,
    allow_untrusted: bool,
) -> anyhow::Result<CertFacts> {
    let addrs = if allow_private_ips {
        resolve_addrs(host, port).await?
    } else {
        resolve_public_addrs(host, port)
            .await
            .inspect_err(warn_probe_private_host)?
    };
    let stream = TcpStream::connect(addrs.as_slice()).await?;
    let config = no_verify_config().ok_or_else(|| anyhow::anyhow!("failed to build TLS config"))?;
    let connector = TlsConnector::from(config);
    let server_name = ServerName::try_from(host.to_owned())?;
    let tls = connector.connect(server_name.clone(), stream).await?;
    let (_io, conn) = tls.get_ref();
    let certs = conn
        .peer_certificates()
        .ok_or_else(|| anyhow::anyhow!("no peer certificates"))?;
    let (leaf, intermediates) = certs
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("empty certificate chain"))?;
    let expires_at = cert_expiry_from_der(leaf.as_ref())
        .ok_or_else(|| anyhow::anyhow!("could not read certificate notAfter"))?;
    let trusted = !allow_untrusted && is_cert_trusted(leaf, intermediates, &server_name);
    if !allow_untrusted && !trusted {
        tracing::warn!(
            "Certificate for '{host}' is untrusted (self-signed or unknown issuer). Tick 'Allow untrusted' on this site if that is expected"
        );
    }
    Ok(CertFacts {
        expires_at,
        trusted,
    })
}

fn is_cert_trusted(
    leaf: &CertificateDer<'_>,
    intermediates: &[CertificateDer<'_>],
    server_name: &ServerName<'_>,
) -> bool {
    trust_verifier().is_some_and(|verifier| {
        verifier
            .verify_server_cert(leaf, intermediates, server_name, &[], UnixTime::now())
            .is_ok()
    })
}

fn cert_expiry_from_der(der: &[u8]) -> Option<DateTime<Utc>> {
    let cert = Certificate::from_der(der).ok()?;
    let secs = cert
        .tbs_certificate
        .validity
        .not_after
        .to_unix_duration()
        .as_secs();
    DateTime::from_timestamp(i64::try_from(secs).ok()?, 0)
}

fn no_verify_config() -> Option<Arc<ClientConfig>> {
    static CONFIG: OnceLock<Option<Arc<ClientConfig>>> = OnceLock::new();
    CONFIG.get_or_init(build_no_verify_config).clone()
}

fn build_no_verify_config() -> Option<Arc<ClientConfig>> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    let config = ClientConfig::builder_with_provider(Arc::clone(&provider))
        .with_safe_default_protocol_versions()
        .ok()?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier { provider }))
        .with_no_client_auth();
    Some(Arc::new(config))
}

fn trust_verifier() -> Option<&'static rustls_platform_verifier::Verifier> {
    static VERIFIER: OnceLock<Option<Arc<rustls_platform_verifier::Verifier>>> = OnceLock::new();
    VERIFIER.get_or_init(build_trust_verifier).as_deref()
}

fn build_trust_verifier() -> Option<Arc<rustls_platform_verifier::Verifier>> {
    let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
    match rustls_platform_verifier::Verifier::new(provider) {
        Ok(verifier) => Some(Arc::new(verifier)),
        Err(e) => {
            tracing::error!("Failed to build platform certificate verifier: {e}");
            None
        }
    }
}

#[derive(Debug)]
struct NoVerifier {
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        clippy::arithmetic_side_effects,
        clippy::unnecessary_wraps
    )]
    use super::*;
    use chrono::Duration as ChronoDuration;

    const EXPIRY_WINDOWS: CertExpiryWindows = CertExpiryWindows {
        warn_days: 30,
        critical_days: 7,
    };

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).expect("valid fixed timestamp")
    }

    fn in_days(days: i64) -> Option<DateTime<Utc>> {
        Some(now() + ChronoDuration::days(days))
    }

    #[test]
    fn trusted_far_future_is_valid() {
        assert_eq!(
            classify_cert_status(in_days(90), true, false, now(), EXPIRY_WINDOWS),
            CertStatus::Valid,
            "trusted cert well clear of expiry should be valid"
        );
    }

    #[test]
    fn within_warn_window_is_expiring() {
        assert_eq!(
            classify_cert_status(in_days(20), true, false, now(), EXPIRY_WINDOWS),
            CertStatus::Expiring,
            "20 days left (<=30) should be expiring"
        );
    }

    #[test]
    fn within_critical_window_is_critical() {
        assert_eq!(
            classify_cert_status(in_days(3), true, false, now(), EXPIRY_WINDOWS),
            CertStatus::Critical,
            "3 days left (<=7) should be critical"
        );
    }

    #[test]
    fn past_expiry_is_expired_regardless_of_trust() {
        assert_eq!(
            classify_cert_status(in_days(-1), true, true, now(), EXPIRY_WINDOWS),
            CertStatus::Expired,
            "an already-expired cert is Expired even when allow_untrusted is set"
        );
    }

    #[test]
    fn past_expiry_beats_untrusted_when_disallowed() {
        assert_eq!(
            classify_cert_status(in_days(-1), false, false, now(), EXPIRY_WINDOWS),
            CertStatus::Expired,
            "expiry is checked before trust: an expired untrusted cert is Expired, not Invalid"
        );
    }

    #[test]
    fn untrusted_beats_imminent_expiry_when_disallowed() {
        assert_eq!(
            classify_cert_status(in_days(3), false, false, now(), EXPIRY_WINDOWS),
            CertStatus::Invalid,
            "untrusted and disallowed is Invalid, taking precedence over the Critical (imminent-expiry) band"
        );
    }

    #[test]
    fn untrusted_judged_on_expiry_when_allowed() {
        assert_eq!(
            classify_cert_status(in_days(90), false, true, now(), EXPIRY_WINDOWS),
            CertStatus::Valid,
            "allow_untrusted on: trust ignored, judged on expiry"
        );
        assert_eq!(
            classify_cert_status(in_days(3), false, true, now(), EXPIRY_WINDOWS),
            CertStatus::Critical,
            "allow_untrusted on: still flags imminent expiry"
        );
    }

    #[test]
    fn missing_expiry_is_invalid() {
        assert_eq!(
            classify_cert_status(None, true, true, now(), EXPIRY_WINDOWS),
            CertStatus::Invalid,
            "no readable expiry should be invalid"
        );
    }

    #[tokio::test]
    async fn cert_check_blocks_private_ip_when_disallowed() {
        let result = check_certificate(
            "https://127.0.0.1/",
            false,
            false,
            Duration::from_secs(1),
            now(),
            EXPIRY_WINDOWS,
        )
        .await;
        assert_eq!(
            result.status,
            CertStatus::Invalid,
            "a host resolving to a private IP must be blocked when private IPs are disallowed"
        );
        assert!(
            result.expires_at.is_none(),
            "a blocked cert check must not report an expiry"
        );
    }
}
