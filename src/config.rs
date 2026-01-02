use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub api_key: String,
    pub database_path: String,
    pub server_port: u16,
    pub check_interval_secs: u64,
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
            check_interval_secs: env::var("CHECK_INTERVAL_SECS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(60),
        }
    }
}

pub const CANARY_URL: &str = "https://www.google.com";
pub const CANARY_TIMEOUT_SECS: u64 = 3;
