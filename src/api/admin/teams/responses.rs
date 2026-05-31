use serde::Serialize;
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Serialize, FromRow, ToSchema)]
pub struct TeamResponse {
    pub id: i64,
    pub name: String,
    pub member_count: i64,
    pub site_count: i64,
}

#[derive(Serialize, FromRow, ToSchema)]
pub struct TeamOption {
    pub id: i64,
    pub name: String,
}
