use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SiteStatus {
    Pending,
    Up,
    Down,
    Blocked,
}

impl SiteStatus {
    pub fn is_up(self) -> bool {
        matches!(self, SiteStatus::Up)
    }

    pub fn is_down(self) -> bool {
        matches!(self, SiteStatus::Down)
    }

    pub fn is_blocked(self) -> bool {
        matches!(self, SiteStatus::Blocked)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CertStatus {
    Valid,
    Expiring,
    Critical,
    Expired,
    Invalid,
    None,
}

#[derive(sqlx::FromRow)]
pub struct SiteRow {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub expected_status: i64,
    pub expected_text: Option<String>,
    pub status: SiteStatus,
    pub tls_allow_untrusted: bool,
}
