// utoipa's OpenApi derive macro triggers this lint in generated code
#![allow(clippy::needless_for_each)]

pub(crate) mod auth;
pub mod errors;
mod healthcheck;
mod pagination;
mod sites;

pub use healthcheck::health;
pub use sites::{SitesApiDoc, routes};

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
          (path = "/api", api = SitesApiDoc),
    ),
    info(
        title = "Site Oxidation API",
        version = "1.0.0",
        description = "API for monitoring sites"
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Bearer,
            )),
        );
    }
}
