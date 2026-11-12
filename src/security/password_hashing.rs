use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

const MAX_WAIT_FOR_HASHING_SLOT: Duration = Duration::from_secs(2);

#[derive(Debug, thiserror::Error)]
pub enum PasswordHashingError {
    #[error("too many passwords are being checked at once")]
    Busy,

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
}

#[derive(Clone)]
pub struct PasswordHashingLimiter {
    permits: Arc<Semaphore>,
    max_wait: Duration,
}

impl PasswordHashingLimiter {
    pub fn new(max_concurrent: usize, max_wait: Duration) -> Self {
        Self {
            permits: Arc::new(Semaphore::new(max_concurrent)),
            max_wait,
        }
    }

    pub fn sized_for_this_machine() -> Self {
        let cpus = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        Self::new(cpus, MAX_WAIT_FOR_HASHING_SLOT)
    }

    pub async fn run<T, F>(&self, work: F) -> Result<T, PasswordHashingError>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let waiting_for_permit = Arc::clone(&self.permits).acquire_owned();
        let Ok(Ok(permit)) = tokio::time::timeout(self.max_wait, waiting_for_permit).await else {
            return Err(PasswordHashingError::Busy);
        };
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            work()
        })
        .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    fn start_blocked_work(
        limiter: &PasswordHashingLimiter,
    ) -> (
        tokio::task::JoinHandle<Result<(), PasswordHashingError>>,
        mpsc::Receiver<()>,
        mpsc::Sender<()>,
    ) {
        let (started_tx, started_rx) = mpsc::channel();
        let (finish_tx, finish_rx) = mpsc::channel::<()>();
        let limiter = limiter.clone();
        let request = tokio::spawn(async move {
            limiter
                .run(move || {
                    let _ = started_tx.send(());
                    let _ = finish_rx.recv();
                })
                .await
        });
        (request, started_rx, finish_tx)
    }

    #[tokio::test]
    async fn test_runs_the_work_and_returns_its_result() {
        let limiter = PasswordHashingLimiter::new(1, Duration::from_secs(1));
        assert_eq!(limiter.run(|| 42).await.unwrap(), 42);
        assert_eq!(limiter.permits.available_permits(), 1);
    }

    #[tokio::test]
    async fn test_reports_busy_when_no_permit_frees_up_in_time() {
        let limiter = PasswordHashingLimiter::new(1, Duration::from_millis(20));
        let (request, started, finish) = start_blocked_work(&limiter);
        tokio::task::spawn_blocking(move || started.recv().unwrap())
            .await
            .unwrap();
        assert!(matches!(
            limiter.run(|| ()).await,
            Err(PasswordHashingError::Busy)
        ));
        finish.send(()).unwrap();
        request.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_dropped_request_keeps_its_permit_until_the_work_finishes() {
        let limiter = PasswordHashingLimiter::new(1, Duration::from_secs(5));
        let (request, started, finish) = start_blocked_work(&limiter);
        tokio::task::spawn_blocking(move || started.recv().unwrap())
            .await
            .unwrap();
        request.abort();
        assert!(request.await.unwrap_err().is_cancelled());
        assert_eq!(
            limiter.permits.available_permits(),
            0,
            "the work is still running, so its permit is still held"
        );
        finish.send(()).unwrap();
        assert_eq!(limiter.run(|| 7).await.unwrap(), 7);
    }
}
