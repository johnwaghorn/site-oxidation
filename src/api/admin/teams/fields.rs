use crate::api::text;
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema)]
#[schema(value_type = String, example = "Platform Team")]
pub struct TeamName(String);

impl TeamName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl<'de> Deserialize<'de> for TeamName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let team_name = text::required(&String::deserialize(deserializer)?, "team name", 60)
            .map_err(D::Error::custom)?;
        Ok(TeamName(team_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("Platform Team", true)]
    #[case(" Platform Team ", true)]
    #[case("A", true)]
    #[case(&"a".repeat(60), true)]
    #[case("", false)]
    #[case("   ", false)]
    #[case(&"a".repeat(61), false)]
    fn test_team_name(#[case] name: &str, #[case] valid: bool) {
        let result: Result<TeamName, _> = serde_json::from_value(json!(name));
        assert_eq!(result.is_ok(), valid);
    }
}
