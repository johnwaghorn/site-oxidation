use axum::body::Body;
use http_body_util::BodyExt;

pub async fn parse_json_body(response: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}
