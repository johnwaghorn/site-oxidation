// utoipa's OpenApi derive macro triggers this lint in generated code
#![allow(clippy::needless_for_each)]

pub(crate) mod admin;
pub(crate) mod auth;
mod auth_queries;
pub mod errors;
pub(crate) mod extractors;
pub(crate) mod healthcheck;
mod pagination;
pub(crate) mod setup;
mod setup_queries;
pub(crate) mod sites;
pub use sites::SitesApiDoc;

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
          (path = "/api", api = SitesApiDoc),
          (path = "/api", api = admin::AdminApiDoc),
          (path = "/api", api = auth::AuthApiDoc),
          (path = "/api", api = setup::SetupApiDoc),
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
            "session_cookie",
            utoipa::openapi::security::SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Cookie(
                    utoipa::openapi::security::ApiKeyValue::new("id"),
                ),
            ),
        );
    }
}
