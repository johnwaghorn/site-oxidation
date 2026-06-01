use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use password_auth::generate_hash;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::task;

use super::queries;
use super::responses::{BootstrapResponse, SetupStatus};
use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::config::AppConfig;
use crate::security::ip::is_private_ip;

#[utoipa::path(
    get,
    path = "/setup/status",
    responses(
        (status = 200, description = "Setup status", body = SetupStatus),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "setup",
)]
pub async fn status(State(pool): State<SqlitePool>) -> Result<Json<SetupStatus>, ApiErrorResponse> {
    let count: i64 = sqlx::query_scalar(queries::COUNT_USERS)
        .fetch_one(&pool)
        .await
        .map_err(|e| internal_err("Failed to check setup status", e))?;
    Ok(Json(SetupStatus {
        setup_required: count == 0,
    }))
}

#[utoipa::path(
    post,
    path = "/setup/bootstrap",
    responses(
        (status = 201, description = "Admin user created", body = BootstrapResponse),
        (status = 403, description = "Not from trusted network", body = ApiError),
        (status = 409, description = "Setup already completed", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError),
    ),
    tag = "setup",
)]
pub async fn bootstrap(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(pool): State<SqlitePool>,
    State(config): State<AppConfig>,
) -> Result<(StatusCode, Json<BootstrapResponse>), ApiErrorResponse> {
    let client_ip = addr.ip();
    let is_trusted = is_private_ip(&client_ip) || config.bootstrap_trusted_ips.contains(&client_ip);
    tracing::info!("Bootstrap attempt from IP: {client_ip} (trusted: {is_trusted})");
    if config.bootstrap_require_private_ip && !is_trusted {
        tracing::warn!(
            client_ip = %client_ip,
            "Rejected bootstrap attempt from an untrusted IP. Connect from a private network, add the IP to BOOTSTRAP_TRUSTED_IPS, or set BOOTSTRAP_REQUIRE_PRIVATE_IP=false if public bootstrap access is intentional."
        );
        return Err(ApiErrorResponse::forbidden(
            "Bootstrap is restricted to local/private networks. Connect from a private network, add your IP to BOOTSTRAP_TRUSTED_IPS, or set BOOTSTRAP_REQUIRE_PRIVATE_IP=false if public bootstrap access is intentional.",
        ));
    }
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| internal_err("Failed to acquire database connection", e))?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(|e| internal_err("Failed to begin bootstrap transaction", e))?;
    let result = do_bootstrap(&mut conn).await;
    if result.is_err() {
        sqlx::query("ROLLBACK").execute(&mut *conn).await.ok();
    } else {
        sqlx::query("COMMIT")
            .execute(&mut *conn)
            .await
            .map_err(|e| internal_err("Failed to commit bootstrap transaction", e))?;
    }
    result
}

async fn do_bootstrap(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
) -> Result<(StatusCode, Json<BootstrapResponse>), ApiErrorResponse> {
    let count: i64 = sqlx::query_scalar(queries::COUNT_USERS)
        .fetch_one(&mut **conn)
        .await
        .map_err(|e| internal_err("Failed to check user count during bootstrap", e))?;
    if count > 0 {
        return Err(ApiErrorResponse::conflict("Setup already completed"));
    }
    let password = generate_random_password()?;
    let password_for_hash = password.clone();
    let hash = task::spawn_blocking(move || generate_hash(&password_for_hash))
        .await
        .map_err(|e| internal_err("Failed to hash bootstrap password", e))?;
    sqlx::query(queries::INSERT_ADMIN)
        .bind(&hash)
        .execute(&mut **conn)
        .await
        .map_err(|e| internal_err("Failed to create admin user", e))?;
    tracing::info!("Bootstrap complete: admin user created");
    Ok((
        StatusCode::CREATED,
        Json(BootstrapResponse {
            username: "admin".to_owned(),
            password,
        }),
    ))
}

fn generate_random_password() -> Result<String, ApiErrorResponse> {
    use std::fmt::Write;
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| internal_err("Failed to generate random bytes", e))?;
    let mut hex = String::with_capacity(64);
    for byte in bytes {
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}
