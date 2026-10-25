use axum::body::Body;
use http_body_util::BodyExt;

pub async fn parse_json_body(response: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

pub fn site_canary_warnings(body: &serde_json::Value) -> Vec<(&str, Option<&str>)> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|site| {
            (
                site["name"].as_str().unwrap(),
                site["canary_warning"].as_str(),
            )
        })
        .collect()
}
