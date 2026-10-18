#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::endpoints;
use super::responses::{ComponentHealth, HealthChecks, HealthStatus, HealthcheckResponse};

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::health),
    components(schemas(HealthcheckResponse, HealthChecks, ComponentHealth, HealthStatus)),
    tags(
        (name = "health", description = "Service health"),
    ),
)]
pub struct HealthApiDoc;
