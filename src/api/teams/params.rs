use crate::api::pagination::deserialize_u32_params;
use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[allow(dead_code)]
pub struct ListTeamNotificationSummariesParams {
    pub sends_alerts: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListTeamNotificationSummariesQuery {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub sends_alerts: Option<bool>,
}
