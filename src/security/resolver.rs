use super::ip::is_private_ip;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("'{host}' resolves to a private/internal IP")]
    PrivateIp { host: String },
    #[error(transparent)]
    Lookup(#[from] std::io::Error),
}

pub async fn resolve_public_addrs(
    host: &str,
    port: u16,
    allow_private: bool,
) -> Result<Vec<SocketAddr>, ResolveError> {
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host((host, port)).await?.collect();
    if !allow_private && addrs.iter().any(|addr| is_private_ip(&addr.ip())) {
        tracing::warn!(
            "Blocked '{host}': resolves to private/internal IP (set PROBE_ALLOW_PRIVATE_IPS=true to allow)"
        );
        return Err(ResolveError::PrivateIp {
            host: host.to_owned(),
        });
    }
    Ok(addrs)
}

pub struct SafeResolver {
    pub allow_private: bool,
}

impl Resolve for SafeResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let allow_private = self.allow_private;
        Box::pin(async move {
            // Port 0: we only need IP resolution, not a specific port.
            let addrs = resolve_public_addrs(name.as_str(), 0, allow_private).await?;
            let boxed: Addrs = Box::new(addrs.into_iter());
            Ok(boxed)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn private_ip_rejection_is_logged() {
        let result = resolve_public_addrs("127.0.0.1", 443, false).await;
        assert!(matches!(result, Err(ResolveError::PrivateIp { .. })));
        assert!(logs_contain(
            "Blocked '127.0.0.1': resolves to private/internal IP"
        ));
        assert!(logs_contain("PROBE_ALLOW_PRIVATE_IPS=true"));
    }
}
