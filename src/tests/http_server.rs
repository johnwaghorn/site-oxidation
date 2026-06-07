use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct TestHttpServer {
    base_url: String,
    port: u16,
    request_count: Arc<AtomicUsize>,
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
        let handle = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 2048];
                let read = socket.read(&mut buf).await.unwrap_or_default();
                let request = String::from_utf8_lossy(&buf[..read]);
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1));
                if path != ignored_path.as_deref() {
                    request_count_for_server.fetch_add(1, Ordering::Relaxed);
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
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}
