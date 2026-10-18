use axum::extract::State;
use axum::{Json, http::StatusCode};
use sqlx::SqlitePool;
use std::time::Instant;

use super::responses::{ComponentHealth, HealthChecks, HealthStatus, HealthcheckResponse};

const DATABASE_UNAVAILABLE_MESSAGE: &str = "Database unavailable";

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "All components healthy", body = HealthcheckResponse),
        (status = 503, description = "One or more components unhealthy", body = HealthcheckResponse),
    ),
    tag = "health",
)]
pub async fn health(State(pool): State<SqlitePool>) -> (StatusCode, Json<HealthcheckResponse>) {
    let db_check_start_time = Instant::now();
    let db_result = sqlx::query("SELECT 1").fetch_one(&pool).await;
    let db_latency = u64::try_from(db_check_start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
    let database = match db_result {
        Ok(_) => ComponentHealth {
            status: HealthStatus::Healthy,
            latency_ms: Some(db_latency),
            error: None,
        },
        Err(e) => {
            tracing::error!("Healthcheck database query failed: {e}");
            ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: None,
                error: Some(DATABASE_UNAVAILABLE_MESSAGE.to_owned()),
            }
        }
    };
    let overall = database.status;
    let status_code = match overall {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    let response = HealthcheckResponse {
        status: overall,
        checks: HealthChecks { database },
    };
    (status_code, Json(response))
}
