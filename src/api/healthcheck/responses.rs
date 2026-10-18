use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct HealthcheckResponse {
    pub status: &'static str,
    pub checks: HealthChecks,
}

#[derive(Serialize, ToSchema)]
pub struct HealthChecks {
    pub database: ComponentHealth,
}

#[derive(Serialize, ToSchema)]
pub struct ComponentHealth {
    pub status: &'static str,
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
