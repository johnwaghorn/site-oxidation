use crate::api::text;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "john")]
pub struct Username(String);

impl Username {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for Username {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let username = text::required(&String::deserialize(deserializer)?, "username", 60)
            .map_err(D::Error::custom)?;
        Ok(Username(username))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("john", true)]
    #[case(" john ", true)]
    #[case("j", true)]
    #[case(&"a".repeat(60), true)]
    #[case("", false)]
    #[case("   ", false)]
    #[case(&"a".repeat(61), false)]
    fn test_username(#[case] username: &str, #[case] valid: bool) {
        let result: Result<Username, _> = serde_json::from_value(json!(username));
        assert_eq!(result.is_ok(), valid);
    }
}
