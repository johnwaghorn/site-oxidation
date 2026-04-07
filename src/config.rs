use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct AppConfig {
    pub allowed_origin: Option<String>,
    pub bootstrap_require_private_ip: bool,
    pub bootstrap_trusted_ips: Vec<std::net::IpAddr>,
    pub canary_timeout_secs: u64,
    pub canary_url: String,
    pub cookie_secure: bool,
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub enable_swagger_ui: bool,
    pub probe_allow_private_ips: bool,
    pub probe_max_concurrent_checks: usize,
    pub probe_retry_count: u32,
    pub probe_retry_delay_ms: u64,
    pub probe_timeout_secs: u64,
    pub probe_user_agent: String,
    pub server_port: u16,
    pub session_key_path: PathBuf,
}

impl AppConfig {
    #[allow(clippy::too_many_lines)]
    pub fn from_env() -> Result<Self> {
        let allowed_origin = env::var("CORS_ALLOWED_ORIGIN").ok();
        let bootstrap_require_private_ip = match env::var("BOOTSTRAP_REQUIRE_PRIVATE_IP") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid BOOTSTRAP_REQUIRE_PRIVATE_IP value: {v}"))?,
            Err(_) => true,
        };
        let bootstrap_trusted_ips = match env::var("BOOTSTRAP_TRUSTED_IPS") {
            Ok(v) => v
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse::<std::net::IpAddr>()
                        .with_context(|| format!("Invalid IP in BOOTSTRAP_TRUSTED_IPS: {s}"))
                })
                .collect::<Result<Vec<_>>>()?,
            Err(_) => Vec::new(),
        };
        let canary_timeout_secs = match env::var("CANARY_TIMEOUT_SECS") {
            Ok(v) => v
                .parse::<u64>()
                .with_context(|| format!("Invalid CANARY_TIMEOUT_SECS value: {v}"))?,
            Err(_) => 3,
        };
        let canary_url = match env::var("CANARY_URL") {
            Ok(v) => v,
            Err(_) => "https://www.google.com".to_owned(),
        };
        let cookie_secure = match env::var("COOKIE_SECURE") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid COOKIE_SECURE value: {v}"))?,
            Err(_) => true,
        };
        let data_dir = PathBuf::from(match env::var("DATA_DIR") {
            Ok(v) => v,
            Err(_) => "./data".to_owned(),
        });
        let database_path = match env::var("DATABASE_PATH") {
            Ok(v) => PathBuf::from(v),
            Err(env::VarError::NotPresent) => data_dir.join("site-oxidation.db"),
            Err(env::VarError::NotUnicode(_)) => {
                anyhow::bail!("DATABASE_PATH must be valid UTF-8");
            }
        };
        let enable_swagger_ui = match env::var("ENABLE_SWAGGER_UI") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid ENABLE_SWAGGER_UI value: {v}"))?,
            Err(_) => false,
        };
        let probe_allow_private_ips = match env::var("PROBE_ALLOW_PRIVATE_IPS") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid PROBE_ALLOW_PRIVATE_IPS value: {v}"))?,
            Err(_) => false,
        };
        let probe_max_concurrent_checks = match env::var("PROBE_MAX_CONCURRENT_CHECKS") {
            Ok(v) => v
                .parse::<usize>()
                .with_context(|| format!("Invalid PROBE_MAX_CONCURRENT_CHECKS value: {v}"))?,
            Err(_) => 20,
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
        let probe_timeout_secs = match env::var("PROBE_TIMEOUT_SECS") {
            Ok(v) => v
                .parse::<u64>()
                .with_context(|| format!("Invalid PROBE_TIMEOUT_SECS value: {v}"))?,
            Err(_) => 30,
        };
        let probe_user_agent = match env::var("PROBE_USER_AGENT") {
            Ok(v) => v,
            Err(_) => {
                "SiteOxidation/1.0 (+https://github.com/johnwaghorn/site-oxidation)".to_owned()
            }
        };
        let server_port = match env::var("SERVER_PORT") {
            Ok(v) => v
                .parse::<u16>()
                .with_context(|| format!("Invalid SERVER_PORT value: {v}"))?,
            Err(_) => 8080,
        };
        let session_key_path = data_dir.join("session.key");

        Ok(Self {
            allowed_origin,
            bootstrap_require_private_ip,
            bootstrap_trusted_ips,
            canary_timeout_secs,
            canary_url,
            cookie_secure,
            data_dir,
            database_path,
            enable_swagger_ui,
            probe_allow_private_ips,
            probe_max_concurrent_checks,
            probe_retry_count,
            probe_retry_delay_ms,
            probe_timeout_secs,
            probe_user_agent,
            server_port,
            session_key_path,
        })
    }
}
