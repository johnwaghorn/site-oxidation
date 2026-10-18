#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::responses::{ExportedSite, SiteExportResponse};

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::export_sites),
    components(schemas(SiteExportResponse, ExportedSite, ApiError)),
    tags(
        (name = "import-export", description = "Move sites between instances"),
    ),
)]
pub struct ImportExportApiDoc;
