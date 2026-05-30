#![allow(clippy::needless_for_each)]

use crate::api::errors::ApiError;
use crate::api::pagination::PaginatedResponse;
use crate::models::site::{CertStatus, SiteStatus};
use utoipa::OpenApi;

use super::endpoints;
use super::requests::SitePayload;
use super::responses::{OutageResponse, SiteResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        endpoints::list_sites,
        endpoints::get_site,
        endpoints::create_site,
        endpoints::update_site,
        endpoints::delete_site,
        endpoints::get_site_outages
    ),
    components(schemas(
        SiteResponse,
        PaginatedResponse<SiteResponse>,
        OutageResponse,
        PaginatedResponse<OutageResponse>,
        SitePayload,
        SiteStatus,
        CertStatus,
        ApiError,
    )),
    tags(
        (name = "sites", description = "Site monitoring"),
    ),
)]
pub struct SitesApiDoc;
