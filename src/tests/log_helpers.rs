use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct TestLogWriter(Arc<Mutex<Vec<u8>>>);

impl TestLogWriter {
    pub fn output(&self) -> String {
        let output = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        String::from_utf8_lossy(&output).into_owned()
    }
}

impl Write for TestLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn capture_warn_logs() -> (TestLogWriter, tracing::dispatcher::DefaultGuard) {
    let logs = TestLogWriter::default();
    let log_writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .without_time()
        .with_ansi(false)
        .with_writer(move || log_writer.clone())
        .finish();
    let guard = tracing::subscriber::set_default(subscriber);
    (logs, guard)
}
