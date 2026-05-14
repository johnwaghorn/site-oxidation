#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::responses::{BootstrapResponse, SetupStatus};

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::status, endpoints::bootstrap),
    components(schemas(SetupStatus, BootstrapResponse, ApiError)),
    tags(
        (name = "setup", description = "First-run setup"),
    ),
)]
pub struct SetupApiDoc;
