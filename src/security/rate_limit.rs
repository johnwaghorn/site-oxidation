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

    pub fn try_acquire(&self, key: &str) -> Option<u32> {
        let mut entry = self
            .attempts
            .entry(key.to_owned())
            .or_insert((0, Instant::now()));
        let (count, started_at) = entry.value_mut();
        if started_at.elapsed() > self.window {
            *count = 0;
            *started_at = Instant::now();
        }
        if *count >= self.max_attempts {
            return None;
        }
        *count = count.saturating_add(1);
        Some(self.max_attempts.saturating_sub(*count))
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
    use std::sync::Arc;

    #[test]
    fn test_allows_attempts_up_to_the_limit() {
        let limiter = LoginRateLimiter::new(3, Duration::from_mins(1));
        let key = "127.0.0.1:admin";
        assert_eq!(limiter.try_acquire(key), Some(2));
        assert_eq!(limiter.try_acquire(key), Some(1));
        assert_eq!(limiter.try_acquire(key), Some(0));
        assert_eq!(limiter.try_acquire(key), None);
    }

    #[test]
    fn test_simultaneous_attempts_cannot_exceed_the_limit() {
        let limiter = Arc::new(LoginRateLimiter::new(5, Duration::from_mins(1)));
        let attempts: Vec<_> = (0..50)
            .map(|_| {
                let limiter = Arc::clone(&limiter);
                std::thread::spawn(move || limiter.try_acquire("127.0.0.1:admin").is_some())
            })
            .collect();
        let allowed = attempts
            .into_iter()
            .filter_map(|attempt| attempt.join().ok())
            .filter(|allowed| *allowed)
            .count();
        assert_eq!(allowed, 5);
    }

    #[test]
    fn test_rejected_attempt_changes_neither_the_count_nor_the_window() {
        let limiter = LoginRateLimiter::new(1, Duration::from_mins(1));
        let key = "127.0.0.1:admin";
        assert!(limiter.try_acquire(key).is_some());
        let before = *limiter.attempts.get(key).unwrap();
        assert!(limiter.try_acquire(key).is_none());
        let after = *limiter.attempts.get(key).unwrap();
        assert_eq!(after, before);
    }

    #[test]
    fn test_expired_window_allows_attempts_again() {
        let limiter = LoginRateLimiter::new(1, Duration::from_millis(1));
        let key = "127.0.0.1:admin";
        assert!(limiter.try_acquire(key).is_some());
        assert!(limiter.try_acquire(key).is_none());
        std::thread::sleep(Duration::from_millis(2));
        assert!(limiter.try_acquire(key).is_some());
    }

    #[test]
    fn test_clear_resets() {
        let limiter = LoginRateLimiter::new(1, Duration::from_mins(1));
        let key = "127.0.0.1:admin";
        assert!(limiter.try_acquire(key).is_some());
        assert!(limiter.try_acquire(key).is_none());
        limiter.clear(key);
        assert!(limiter.try_acquire(key).is_some());
    }

    #[test]
    fn test_independent_keys() {
        let limiter = LoginRateLimiter::new(1, Duration::from_mins(1));
        assert!(limiter.try_acquire("ip1:admin").is_some());
        assert!(limiter.try_acquire("ip1:admin").is_none());
        assert!(limiter.try_acquire("ip2:admin").is_some());
    }

    #[test]
    fn test_prune_expired_removes_stale_entries() {
        let limiter = LoginRateLimiter::new(3, Duration::from_millis(1));
        limiter.try_acquire("ip1:admin");
        limiter.try_acquire("ip2:user");
        std::thread::sleep(Duration::from_millis(2));
        limiter.prune_expired();
        assert!(limiter.attempts.is_empty());
    }
}
