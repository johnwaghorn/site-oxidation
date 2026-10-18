use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[derive(Serialize, ToSchema)]
pub struct HealthcheckResponse {
    pub status: HealthStatus,
    pub checks: HealthChecks,
}

#[derive(Serialize, ToSchema)]
pub struct HealthChecks {
    pub database: ComponentHealth,
}

#[derive(Serialize, ToSchema)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
