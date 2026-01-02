use axum::extract::State;
use axum::{Json, http::StatusCode};
use serde::Serialize;
use sqlx::SqlitePool;
use std::time::Instant;

#[derive(Serialize)]
pub struct HealthcheckResponse {
    status: &'static str,
    version: &'static str,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: ComponentHealth,
}

#[derive(Serialize)]
pub struct ComponentHealth {
    status: &'static str,
    latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub async fn health(State(pool): State<SqlitePool>) -> (StatusCode, Json<HealthcheckResponse>) {
    let db_check_start_time = Instant::now();
    let db_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    let db_latency = u64::try_from(db_check_start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
    let database = match db_result {
        Ok(_) => ComponentHealth {
            status: "healthy",
            latency_ms: Some(db_latency),
            error: None,
        },
        Err(e) => ComponentHealth {
            status: "unhealthy",
            latency_ms: None,
            error: Some(e.to_string()),
        },
    };
    let all_components_healthy = database.status == "healthy";
    let response = HealthcheckResponse {
        status: if all_components_healthy {
            "healthy"
        } else {
            "unhealthy"
        },
        version: env!("CARGO_PKG_VERSION"),
        checks: HealthChecks { database },
    };
    let status = if all_components_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(response))
}
