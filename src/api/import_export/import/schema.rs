#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use utoipa::OpenApi;

use super::endpoints;
use super::requests::{ImportedSite, SiteImportRequest};
use super::responses::{ImportOutcome, ImportedSiteResult, SiteImportResponse};

#[derive(OpenApi)]
#[openapi(
    paths(endpoints::import_sites),
    components(schemas(
        SiteImportRequest,
        ImportedSite,
        SiteImportResponse,
        ImportedSiteResult,
        ImportOutcome,
        ApiError,
    ))
)]
pub struct ImportApiDoc;
