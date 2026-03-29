use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub session_key_path: PathBuf,
    pub server_port: u16,
    pub probe_timeout_secs: u64,
    pub probe_retry_count: u32,
    pub probe_retry_delay_ms: u64,
    pub allow_private_ips: bool,
    pub cookie_secure: bool,
    pub allowed_origin: Option<String>,
    pub enable_swagger_ui: bool,
    pub user_agent: String,
    pub canary_url: String,
    pub canary_timeout_secs: u64,
    pub probe_max_concurrent_checks: usize,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
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
        let session_key_path = data_dir.join("session.key");
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
        let cookie_secure = match env::var("COOKIE_SECURE") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid COOKIE_SECURE value: {v}"))?,
            Err(_) => true,
        };
        let allowed_origin = env::var("ALLOWED_ORIGIN").ok();
        let enable_swagger_ui = match env::var("ENABLE_SWAGGER_UI") {
            Ok(v) => v
                .parse::<bool>()
                .with_context(|| format!("Invalid ENABLE_SWAGGER_UI value: {v}"))?,
            Err(_) => false,
        };
        let user_agent = match env::var("USER_AGENT") {
            Ok(v) => v,
            Err(_) => {
                "SiteOxidation/1.0 (+https://github.com/johnwaghorn/site-oxidation)".to_owned()
            }
        };
        let canary_url = match env::var("CANARY_URL") {
            Ok(v) => v,
            Err(_) => "https://www.google.com".to_owned(),
        };
        let canary_timeout_secs = match env::var("CANARY_TIMEOUT_SECS") {
            Ok(v) => v
                .parse::<u64>()
                .with_context(|| format!("Invalid CANARY_TIMEOUT_SECS value: {v}"))?,
            Err(_) => 3,
        };
        let probe_max_concurrent_checks = match env::var("PROBE_MAX_CONCURRENT_CHECKS") {
            Ok(v) => v
                .parse::<usize>()
                .with_context(|| format!("Invalid PROBE_MAX_CONCURRENT_CHECKS value: {v}"))?,
            Err(_) => 20,
        };

        Ok(Self {
            data_dir,
            database_path,
            session_key_path,
            server_port,
            probe_timeout_secs,
            probe_retry_count,
            probe_retry_delay_ms,
            allow_private_ips,
            cookie_secure,
            allowed_origin,
            enable_swagger_ui,
            user_agent,
            canary_url,
            canary_timeout_secs,
            probe_max_concurrent_checks,
        })
    }
}
