use crate::security::ip::is_private_ip;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::net::SocketAddr;
use thiserror::Error;

#[derive(Debug, Error)]
enum ResolverError {
    #[error("resolved to private IP")]
    PrivateIpBlocked,
}

pub struct SafeResolver {
    pub allow_private: bool,
}

impl Resolve for SafeResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let allow_private = self.allow_private;
        Box::pin(async move {
            // Port 0: we only need IP resolution, not a specific port
            let addrs: Vec<SocketAddr> =
                tokio::net::lookup_host((name.as_str(), 0)).await?.collect();
            if !allow_private && addrs.iter().any(|a| is_private_ip(&a.ip())) {
                return Err(ResolverError::PrivateIpBlocked.into());
            }
            let boxed: Addrs = Box::new(addrs.into_iter());
            Ok(boxed)
        })
    }
}
