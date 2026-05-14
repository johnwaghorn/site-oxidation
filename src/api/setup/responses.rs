use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SetupStatus {
    pub setup_required: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BootstrapResponse {
    pub username: String,
    pub password: String,
}
