use crate::error::RpcError;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const DEFAULT_MAX_TRACKED_KEYS: usize = 100_000;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: HashMap<String, VecDeque<Instant>>,
    max_requests: usize,
    window: Duration,
    max_tracked_keys: usize,
}

impl RateLimiter {
    #[must_use]
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window,
            max_tracked_keys: DEFAULT_MAX_TRACKED_KEYS,
        }
    }

    #[must_use]
    pub fn with_limits(max_requests: usize, window: Duration, max_tracked_keys: usize) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window,
            max_tracked_keys,
        }
    }

    pub fn check(&mut self, key: &str) -> Result<(), RpcError> {
        let now = Instant::now();
        self.prune_expired_at(now);
        self.guard_capacity(key);

        let entries = self.requests.entry(key.to_string()).or_default();

        if entries.len() >= self.max_requests {
            let oldest = entries.front().copied().unwrap_or(now);
            let elapsed = now.duration_since(oldest);
            let retry_after = self.window.saturating_sub(elapsed);
            let retry_after_ms = u64::try_from(retry_after.as_millis()).unwrap_or(u64::MAX);
            return Err(RpcError::RateLimitExceeded { retry_after_ms });
        }

        entries.push_back(now);
        Ok(())
    }

    #[must_use]
    pub fn active_key_count(&self) -> usize {
        self.requests.len()
    }

    pub fn prune_expired(&mut self) {
        self.prune_expired_at(Instant::now());
    }

    fn prune_expired_at(&mut self, now: Instant) {
        self.requests.retain(|_, entries| {
            while let Some(oldest) = entries.front().copied() {
                if now.duration_since(oldest) <= self.window {
                    break;
                }

                entries.pop_front();
            }

            !entries.is_empty()
        });
    }

    fn guard_capacity(&mut self, key: &str) {
        if self.requests.contains_key(key) || self.requests.len() < self.max_tracked_keys {
            return;
        }

        if let Some(lru_key) = self.oldest_key() {
            self.requests.remove(&lru_key);
        }
    }

    fn oldest_key(&self) -> Option<String> {
        self.requests
            .iter()
            .filter_map(|(key, entries)| entries.front().map(|oldest| (key, oldest)))
            .min_by_key(|(_, oldest)| **oldest)
            .map(|(key, _)| key.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforces_rate_limit_and_returns_retry_after() {
        let mut limiter = RateLimiter::new(2, Duration::from_millis(200));

        assert!(limiter.check("peer-a").is_ok());
        assert!(limiter.check("peer-a").is_ok());

        let err = limiter
            .check("peer-a")
            .expect_err("third request should be rate-limited");

        match err {
            RpcError::RateLimitExceeded { retry_after_ms } => {
                assert!(retry_after_ms > 0);
                assert!(retry_after_ms <= 200);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn prune_expired_removes_inactive_keys() {
        let mut limiter = RateLimiter::new(1, Duration::from_millis(20));
        assert!(limiter.check("peer-a").is_ok());
        assert!(limiter.check("peer-b").is_ok());
        assert_eq!(limiter.active_key_count(), 2);

        std::thread::sleep(Duration::from_millis(30));
        limiter.prune_expired();

        assert_eq!(limiter.active_key_count(), 0);
    }

    #[test]
    fn evicts_oldest_key_when_capacity_is_reached() {
        let mut limiter = RateLimiter::with_limits(1, Duration::from_secs(5), 2);

        assert!(limiter.check("peer-a").is_ok());
        std::thread::sleep(Duration::from_millis(2));
        assert!(limiter.check("peer-b").is_ok());

        assert_eq!(limiter.active_key_count(), 2);

        assert!(limiter.check("peer-c").is_ok());
        assert_eq!(limiter.active_key_count(), 2);

        assert!(limiter.check("peer-a").is_ok());
    }

    #[test]
    fn existing_key_is_not_evicted_when_capacity_is_reached() {
        let mut limiter = RateLimiter::with_limits(2, Duration::from_secs(5), 1);

        assert!(limiter.check("peer-a").is_ok());
        assert!(limiter.check("peer-a").is_ok());
        assert_eq!(limiter.active_key_count(), 1);

        let err = limiter
            .check("peer-a")
            .expect_err("same key should be rate-limited instead of evicted");

        assert!(matches!(err, RpcError::RateLimitExceeded { .. }));
        assert_eq!(limiter.active_key_count(), 1);
    }

    #[test]
    fn allows_requests_again_after_window_expires() {
        let mut limiter = RateLimiter::new(1, Duration::from_millis(25));

        assert!(limiter.check("peer-a").is_ok());
        assert!(matches!(
            limiter.check("peer-a"),
            Err(RpcError::RateLimitExceeded { .. })
        ));

        std::thread::sleep(Duration::from_millis(30));

        assert!(limiter.check("peer-a").is_ok());
    }
}
