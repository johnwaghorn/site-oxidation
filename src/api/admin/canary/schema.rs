#![allow(clippy::needless_for_each)]

use super::endpoints;
use super::requests::UpdateCanarySettingsRequest;
use super::responses::CanarySettingsResponse;
use crate::api::errors::ApiError;
use crate::canary::CanaryState;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::get_settings,
        endpoints::update_settings,
        endpoints::test_canary,
    ),
    components(schemas(
        UpdateCanarySettingsRequest,
        CanarySettingsResponse,
        CanaryState,
        ApiError,
    )),
    tags(
        (name = "admin/canary", description = "Global connectivity canary settings (admin only)"),
    ),
)]
pub struct CanaryApiDoc;
