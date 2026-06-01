/* Pagination

Standardised paginated responses for the API. SQL OFFSET is 0-indexed.

Note: Invalid pagination params are coerced into valid fields for improved UX.
 */

use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PaginationParams {
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    #[param(minimum = 1, maximum = 10000, default = 1, example = 1)]
    pub page: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_u32_params")]
    #[param(minimum = 20, maximum = 100, default = 20, example = 20)]
    pub per_page: Option<u32>,
}

pub(crate) fn deserialize_u32_params<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let param = Option::<u64>::deserialize(deserializer)?;
    Ok(match param {
        None => None,
        Some(v) => {
            if let Ok(value) = u32::try_from(v) {
                Some(value)
            } else {
                tracing::warn!("Pagination value {v} exceeds u32 max, using default value");
                None
            }
        }
    })
}

impl PaginationParams {
    const DEFAULT_PAGE: u32 = 1;
    const DEFAULT_PER_PAGE: u32 = 20;
    const MAX_PAGE: u32 = 10000;
    const MAX_PER_PAGE: u32 = 100;

    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Self {
        Self { page, per_page }
    }

    pub fn page(&self) -> u32 {
        self.page
            .unwrap_or(Self::DEFAULT_PAGE)
            .clamp(Self::DEFAULT_PAGE, Self::MAX_PAGE)
    }

    pub fn per_page(&self) -> u32 {
        self.per_page
            .unwrap_or(Self::DEFAULT_PER_PAGE)
            .clamp(Self::DEFAULT_PER_PAGE, Self::MAX_PER_PAGE)
    }

    pub fn offset(&self) -> u32 {
        self.page()
            .saturating_sub(1)
            .saturating_mul(self.per_page())
    }
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;
    #[test]
    fn test_page_clamped_to_min() {
        let params = PaginationParams {
            page: Some(0),
            per_page: None,
        };
        assert_eq!(params.page(), PaginationParams::DEFAULT_PAGE)
    }

    #[test]
    fn test_page_clamped_to_max() {
        let params = PaginationParams {
            page: Some(999999999),
            per_page: None,
        };
        assert_eq!(params.page(), PaginationParams::MAX_PAGE)
    }

    #[test]
    fn test_per_page_clamped_to_min() {
        let params = PaginationParams {
            page: None,
            per_page: Some(0),
        };
        assert_eq!(params.per_page(), PaginationParams::DEFAULT_PER_PAGE)
    }

    #[test]
    fn test_per_page_clamped_to_max() {
        let params = PaginationParams {
            page: None,
            per_page: Some(500),
        };
        assert_eq!(params.per_page(), PaginationParams::MAX_PER_PAGE)
    }

    #[test]
    fn test_offset_first_page() {
        let params = PaginationParams {
            page: Some(1),
            per_page: Some(PaginationParams::DEFAULT_PER_PAGE),
        };
        assert_eq!(params.offset(), 0)
    }

    #[test]
    fn test_offset_second_page() {
        let params = PaginationParams {
            page: Some(2),
            per_page: Some(PaginationParams::DEFAULT_PER_PAGE),
        };
        assert_eq!(params.offset(), 20)
    }

    #[test]
    fn test_openapi_docs_catch_drift() {
        // Important: if this fails, update #[param] attributes to match
        assert_eq!(PaginationParams::DEFAULT_PAGE, 1);
        assert_eq!(PaginationParams::DEFAULT_PER_PAGE, 20);
        assert_eq!(PaginationParams::MAX_PAGE, 10000);
        assert_eq!(PaginationParams::MAX_PER_PAGE, 100);
    }

    #[traced_test]
    #[test]
    fn test_u32_overflow_is_logged_and_uses_default() {
        #[derive(Deserialize)]
        struct OverflowParam {
            #[serde(deserialize_with = "deserialize_u32_params")]
            value: Option<u32>,
        }
        let param: OverflowParam = serde_json::from_value(serde_json::json!({
            "value": u64::from(u32::MAX) + 1,
        }))
        .unwrap();
        assert_eq!(param.value, None);
        assert!(logs_contain("exceeds u32 max, using default value"));
    }
}
