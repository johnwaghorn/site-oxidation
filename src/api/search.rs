use serde::Deserialize;
use utoipa::IntoParams;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SearchParams {
    /// Case-insensitive substring match.
    pub search: Option<String>,
}

impl SearchParams {
    pub fn normalised(&self) -> Option<&str> {
        normalise_search(self.search.as_deref())
    }
}

pub fn normalise_search(search: Option<&str>) -> Option<&str> {
    search.map(str::trim).filter(|search| !search.is_empty())
}
