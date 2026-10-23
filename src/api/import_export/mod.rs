pub mod export;
pub mod import;
pub mod schema;

use axum::Router;

use crate::state::AppState;

const SITE_EXPORT_VERSION: u32 = 1;

pub fn import_export_routes() -> Router<AppState> {
    Router::new()
        .merge(export::export_routes())
        .merge(import::import_routes())
}
