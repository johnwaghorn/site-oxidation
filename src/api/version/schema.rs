#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::responses::VersionResponse;

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::version),
    components(schemas(VersionResponse, ApiError)),
    tags(
        (name = "version", description = "Application version"),
    ),
)]
pub struct VersionApiDoc;
