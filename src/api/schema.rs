use utoipa::OpenApi;

use super::admin::schema::AdminApiDoc;
use super::auth::AuthApiDoc;
use super::setup::SetupApiDoc;
use super::sites::schema::SitesApiDoc;

#[allow(clippy::needless_for_each)]
#[derive(OpenApi)]
#[openapi(
    nest(
          (path = "/api", api = SitesApiDoc),
          (path = "/api", api = AdminApiDoc),
          (path = "/api", api = AuthApiDoc),
          (path = "/api", api = SetupApiDoc),
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
