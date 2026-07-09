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

pub async fn resolve_addrs(host: &str, port: u16) -> Result<Vec<SocketAddr>, std::io::Error> {
    Ok(tokio::net::lookup_host((host, port)).await?.collect())
}

pub async fn resolve_public_addrs(host: &str, port: u16) -> Result<Vec<SocketAddr>, ResolveError> {
    let addrs = resolve_addrs(host, port).await?;
    if addrs.iter().any(|addr| is_private_ip(&addr.ip())) {
        return Err(ResolveError::PrivateIp {
            host: host.to_owned(),
        });
    }
    Ok(addrs)
}

pub fn warn_probe_private_host(error: &ResolveError) {
    if let ResolveError::PrivateIp { host } = error {
        tracing::warn!(
            "Blocked '{host}': resolves to private/internal IP (set PROBE_ALLOW_PRIVATE_IPS=true to allow)"
        );
    }
}

pub struct SafeResolver {
    pub allow_private: bool,
}

impl Resolve for SafeResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let allow_private = self.allow_private;
        Box::pin(async move {
            // Port 0: we only need IP resolution, not a specific port.
            let addrs = if allow_private {
                resolve_addrs(name.as_str(), 0).await?
            } else {
                resolve_public_addrs(name.as_str(), 0)
                    .await
                    .inspect_err(warn_probe_private_host)?
            };
            let boxed: Addrs = Box::new(addrs.into_iter());
            Ok(boxed)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_public_addrs_rejects_private_ips() {
        let result = resolve_public_addrs("127.0.0.1", 443).await;
        assert!(matches!(result, Err(ResolveError::PrivateIp { .. })));
    }

    #[tokio::test]
    async fn resolve_addrs_permits_private_ips() {
        let result = resolve_addrs("127.0.0.1", 443).await;
        assert!(result.is_ok());
    }
}
