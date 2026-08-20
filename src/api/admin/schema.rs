#![allow(clippy::needless_for_each)]

use utoipa::OpenApi;

use super::canary::schema::CanaryApiDoc;
use super::teams::schema::TeamsApiDoc;
use super::users::schema::UsersApiDoc;

#[derive(OpenApi)]
#[openapi(nest(
    (path = "/admin", api = UsersApiDoc),
    (path = "/admin", api = TeamsApiDoc),
    (path = "/admin", api = CanaryApiDoc),
))]
pub struct AdminApiDoc;
