use crate::tests::TEST_API_KEY;

mod test_api_auth;
mod test_api_sites;

pub fn test_auth_header() -> (&'static str, String) {
    ("Authorization", format!("Bearer {}", TEST_API_KEY))
}
