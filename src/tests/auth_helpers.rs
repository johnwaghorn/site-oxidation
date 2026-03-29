use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

pub fn build_login_request(username: &str, password: &str) -> Request<Body> {
    let mut request = Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"username":"{username}","password":"{password}"}}"#
        )))
        .unwrap();
    request
        .extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            super::fixtures::LOOPBACK_IP,
            0,
        ))));
    request
}

pub fn build_change_password_request(cookie: &str, current: &str, new: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/auth/change-password")
        .header("cookie", cookie)
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"current_password":"{current}","new_password":"{new}"}}"#
        )))
        .unwrap()
}

pub async fn login_and_get_cookie(app: &Router, username: &str, password: &str) -> String {
    let response = app
        .clone()
        .oneshot(build_login_request(username, password))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Login failed");
    extract_cookies(&response)
}

fn extract_cookies<T>(response: &axum::http::Response<T>) -> String {
    let mut cookies = String::new();
    for value in response.headers().get_all("set-cookie") {
        if let Ok(s) = value.to_str() {
            if !cookies.is_empty() {
                cookies.push_str("; ");
            }
            if let Some(cookie_pair) = s.split(';').next() {
                cookies.push_str(cookie_pair);
            }
        }
    }
    cookies
}
