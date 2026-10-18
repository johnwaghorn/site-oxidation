#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::endpoints;
use super::responses::{ComponentHealth, HealthChecks, HealthcheckResponse};

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::health),
    components(schemas(HealthcheckResponse, HealthChecks, ComponentHealth)),
    tags(
        (name = "health", description = "Service health"),
    ),
)]
pub struct HealthApiDoc;
