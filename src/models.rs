#[derive(sqlx::FromRow)]
pub struct SiteRow {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub expected_status: i64,
    pub expected_text: Option<String>,
    pub is_up: i64,
}
