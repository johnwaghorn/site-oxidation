use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use password_auth::generate_hash;
use serde::Serialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tokio::task;
use utoipa::{OpenApi, ToSchema};

use crate::api::errors::{ApiError, ApiErrorResponse, internal_err};
use crate::config::AppConfig;
use crate::security::ip::is_private_ip;
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(status, bootstrap),
    components(schemas(SetupStatus, BootstrapResponse, ApiError)),
    tags(
        (name = "setup", description = "First-run setup"),
    ),
)]
pub struct SetupApiDoc;

pub fn setup_routes() -> Router<AppState> {
    Router::new()
        .route("/setup/status", get(status))
        .route("/setup/bootstrap", post(bootstrap))
}

#[derive(Serialize, ToSchema)]
pub struct SetupStatus {
    pub setup_required: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BootstrapResponse {
    pub username: String,
    pub password: String,
}

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
    let count: i64 = sqlx::query_scalar(super::setup_queries::COUNT_USERS)
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
    if config.bootstrap_require_private_ip && !is_private_ip(&addr.ip()) {
        return Err(ApiErrorResponse::forbidden(
            "Bootstrap restricted to local/private network",
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
    let count: i64 = sqlx::query_scalar(super::setup_queries::COUNT_USERS)
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
    sqlx::query(super::setup_queries::INSERT_ADMIN)
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
