use anyhow::{Context, Result};
use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub database_path: String,
    pub server_port: u16,
    pub probe_timeout_secs: u64,
    pub probe_retry_count: u32,
    pub probe_retry_delay_ms: u64,
    pub allow_private_ips: bool,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("API_KEY").context("Please set API_KEY")?;
        let database_path = match env::var("DATABASE_PATH") {
            Ok(v) => v,
            Err(env::VarError::NotPresent) => "./data/site-oxidation.db".to_string(),
            Err(env::VarError::NotUnicode(_)) => {
                anyhow::bail!("DATABASE_PATH must be valid UTF-8");
            }
        };
        let server_port = match env::var("PORT") {
            Ok(v) => v
                .parse::<u16>()
                .with_context(|| format!("Invalid PORT value: {v}"))?,
            Err(_) => 8080,
        };
        let probe_timeout_secs = match env::var("PROBE_TIMEOUT_SECS") {
            Ok(v) => v
                .parse::<u64>()
                .with_context(|| format!("Invalid PROBE_TIMEOUT_SECS value: {v}"))?,
            Err(_) => 30,
        };
        let probe_retry_count = match env::var("PROBE_RETRY_COUNT") {
            Ok(v) => v
                .parse::<u32>()
                .with_context(|| format!("Invalid PROBE_RETRY_COUNT value: {v}"))?,
            Err(_) => 2,
        };
        let probe_retry_delay_ms = match env::var("PROBE_RETRY_DELAY_MS") {
            Ok(v) => v
                .parse::<u64>()
                .with_context(|| format!("Invalid PROBE_RETRY_DELAY_MS value: {v}"))?,
            Err(_) => 3000,
        };
        let allow_private_ips = match env::var("ALLOW_PRIVATE_IPS") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid ALLOW_PRIVATE_IPS value: {v}"))?,
            Err(_) => false,
        };

        Ok(Self {
            api_key,
            database_path,
            server_port,
            probe_timeout_secs,
            probe_retry_count,
            probe_retry_delay_ms,
            allow_private_ips,
        })
    }
}

// Canary check config
pub const CANARY_URL: &str = "https://www.google.com";
pub const CANARY_TIMEOUT_SECS: u64 = 3;

// Probe config
pub const PROBE_MAX_CONCURRENT_CHECKS: usize = 20;
