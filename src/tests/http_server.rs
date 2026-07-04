use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct TestHttpServer {
    base_url: String,
    port: u16,
    request_count: Arc<AtomicUsize>,
    requests: Arc<Mutex<Vec<String>>>,
    handle: JoinHandle<()>,
}

impl TestHttpServer {
    pub async fn start() -> Self {
        Self::start_with_ignored_path(None).await
    }

    pub async fn start_ignoring_path(path: &str) -> Self {
        Self::start_with_ignored_path(Some(path.to_owned())).await
    }

    async fn start_with_ignored_path(ignored_path: Option<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let request_count = Arc::new(AtomicUsize::new(0));
        let request_count_for_server = Arc::clone(&request_count);
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_server = Arc::clone(&requests);
        let handle = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut data = Vec::new();
                let mut buf = [0u8; 2048];
                loop {
                    let read = socket.read(&mut buf).await.unwrap_or_default();
                    if read == 0 {
                        break;
                    }
                    data.extend_from_slice(&buf[..read]);
                    if has_headers_and_full_body(&data) || data.len() > 65536 {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&data).into_owned();
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1));
                if path != ignored_path.as_deref() {
                    request_count_for_server.fetch_add(1, Ordering::Relaxed);
                    requests_for_server.lock().unwrap().push(request);
                }
                let response =
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                let _ = socket.write_all(response).await;
            }
        });
        Self {
            base_url: format!("http://{addr}"),
            port: addr.port(),
            request_count,
            requests,
            handle,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn request_count(&self) -> usize {
        self.request_count.load(Ordering::Relaxed)
    }

    pub fn last_request(&self) -> Option<String> {
        self.requests.lock().unwrap().last().cloned()
    }
}

fn has_headers_and_full_body(data: &[u8]) -> bool {
    let request = String::from_utf8_lossy(data);
    let Some((headers, body)) = request.split_once("\r\n\r\n") else {
        return false;
    };
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse::<usize>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);
    body.len() >= content_length
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
