use crate::api::pagination::deserialize_u32_params;
use serde::Deserialize;
use utoipa::openapi::RefOr;
use utoipa::openapi::schema::{ObjectBuilder, Schema, Type};
use utoipa::{IntoParams, PartialSchema, ToSchema};

pub const UPTIME_WINDOW_DAY_OPTIONS: [u32; 3] = [7, 30, 90];

pub struct UptimeWindowDays;

impl PartialSchema for UptimeWindowDays {
    fn schema() -> RefOr<Schema> {
        ObjectBuilder::new()
            .schema_type(Type::Integer)
            .enum_values(Some(UPTIME_WINDOW_DAY_OPTIONS))
            .into()
    }
}

impl ToSchema for UptimeWindowDays {}

/// Status buckets. `down` also matches blocked sites, since both mean the site needs attention.
#[derive(Debug, Clone, Copy, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SiteStatusFilter {
    Up,
    Down,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListSitesParams {
    /// Restrict results by status. `down` includes blocked sites.
    pub status: Option<SiteStatusFilter>,
}

#[derive(Deserialize)]
pub struct ListSitesQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub status: Option<SiteStatusFilter>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SiteUptimeParams {
    /// Days to cover, ending today.
    #[param(value_type = UptimeWindowDays, example = 90)]
    pub days: u32,
    /// IANA time zone that decides where each day starts.
    #[param(example = "Europe/London")]
    pub time_zone: String,
}
