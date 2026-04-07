use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct LoginRateLimiter {
    attempts: DashMap<String, (u32, Instant)>,
    max_attempts: u32,
    window: Duration,
}

impl LoginRateLimiter {
    pub fn new(max_attempts: u32, window: Duration) -> Self {
        Self {
            attempts: DashMap::new(),
            max_attempts,
            window,
        }
    }

    pub fn window_secs(&self) -> u64 {
        self.window.as_secs()
    }

    pub fn is_blocked(&self, key: &str) -> bool {
        if let Some(entry) = self.attempts.get(key) {
            let (count, started_at) = *entry;
            if started_at.elapsed() > self.window {
                return false;
            }
            count >= self.max_attempts
        } else {
            false
        }
    }

    pub fn record_failure(&self, key: &str) -> bool {
        let mut entry = self
            .attempts
            .entry(key.to_owned())
            .or_insert((0, Instant::now()));
        let (count, started_at) = entry.value_mut();
        if started_at.elapsed() > self.window {
            *count = 1;
            *started_at = Instant::now();
        } else {
            *count = count.saturating_add(1);
        }
        *count >= self.max_attempts
    }

    pub fn clear(&self, key: &str) {
        self.attempts.remove(key);
    }

    pub fn prune_expired(&self) {
        self.attempts
            .retain(|_, (_, started_at)| started_at.elapsed() <= self.window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allows_under_limit() {
        let limiter = LoginRateLimiter::new(3, Duration::from_secs(60));
        let key = "127.0.0.1:admin";
        assert!(!limiter.record_failure(key));
        assert!(!limiter.record_failure(key));
        assert!(!limiter.is_blocked(key));
    }

    #[test]
    fn test_blocks_at_limit() {
        let limiter = LoginRateLimiter::new(3, Duration::from_secs(60));
        let key = "127.0.0.1:admin";
        limiter.record_failure(key);
        limiter.record_failure(key);
        assert!(limiter.record_failure(key));
        assert!(limiter.is_blocked(key));
    }

    #[test]
    fn test_clear_resets() {
        let limiter = LoginRateLimiter::new(3, Duration::from_secs(60));
        let key = "127.0.0.1:admin";
        limiter.record_failure(key);
        limiter.record_failure(key);
        limiter.record_failure(key);
        assert!(limiter.is_blocked(key));
        limiter.clear(key);
        assert!(!limiter.is_blocked(key));
    }

    #[test]
    fn test_expired_window_unblocks() {
        let limiter = LoginRateLimiter::new(3, Duration::from_millis(1));
        let key = "127.0.0.1:admin";
        limiter.record_failure(key);
        limiter.record_failure(key);
        limiter.record_failure(key);
        std::thread::sleep(Duration::from_millis(2));
        assert!(!limiter.is_blocked(key));
    }

    #[test]
    fn test_independent_keys() {
        let limiter = LoginRateLimiter::new(2, Duration::from_secs(60));
        limiter.record_failure("ip1:admin");
        limiter.record_failure("ip1:admin");
        assert!(limiter.is_blocked("ip1:admin"));
        assert!(!limiter.is_blocked("ip2:admin"));
    }

    #[test]
    fn test_prune_expired_removes_stale_entries() {
        let limiter = LoginRateLimiter::new(3, Duration::from_millis(1));
        limiter.record_failure("ip1:admin");
        limiter.record_failure("ip2:user");
        std::thread::sleep(Duration::from_millis(2));
        limiter.prune_expired();
        assert!(!limiter.is_blocked("ip1:admin"));
        assert!(!limiter.is_blocked("ip2:user"));
        assert!(limiter.attempts.is_empty());
    }

    #[test]
    fn test_record_failure_resets_expired_window() {
        let limiter = LoginRateLimiter::new(3, Duration::from_millis(1));
        let key = "127.0.0.1:admin";
        limiter.record_failure(key);
        limiter.record_failure(key);
        limiter.record_failure(key);
        assert!(limiter.is_blocked(key));
        std::thread::sleep(Duration::from_millis(2));
        assert!(!limiter.record_failure(key));
        assert!(!limiter.is_blocked(key));
    }
}
