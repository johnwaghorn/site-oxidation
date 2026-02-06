use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub database_path: String,
    pub server_port: u16,
    pub probe_timeout_secs: u64,
    pub probe_retry_count: u32,
    pub probe_retry_delay_ms: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("API_KEY").expect("API_KEY must be set"),
            database_path: env::var("DATABASE_PATH")
                .unwrap_or_else(|_| "./data/site-oxidation.db".to_string()),
            server_port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            probe_timeout_secs: env::var("PROBE_TIMEOUT_SECS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(30),
            probe_retry_count: env::var("PROBE_RETRY_COUNT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(2),
            probe_retry_delay_ms: env::var("PROBE_RETRY_DELAY_MS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
        }
    }
}

// Canary check config
pub const CANARY_URL: &str = "https://www.google.com";
pub const CANARY_TIMEOUT_SECS: u64 = 3;

// Probe config
pub const PROBE_MAX_CONCURRENT_CHECKS: usize = 20;
