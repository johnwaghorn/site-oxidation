#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::export::schema::ExportApiDoc;
use super::import::schema::ImportApiDoc;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/sites", api = ExportApiDoc),
        (path = "/sites", api = ImportApiDoc),
    ),
    tags(
        (name = "import-export", description = "Move sites between instances"),
    ),
)]
pub struct ImportExportApiDoc;
