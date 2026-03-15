use crate::error::RpcError;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    #[must_use]
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: HashMap::new(),
            max_requests,
            window,
        }
    }

    pub fn check(&mut self, key: &str) -> Result<(), RpcError> {
        let now = Instant::now();
        let entries = self.requests.entry(key.to_string()).or_default();
        entries.retain(|entry| now.duration_since(*entry) <= self.window);

        if entries.len() >= self.max_requests {
            let oldest = entries.iter().min().copied().unwrap_or(now);
            let elapsed = now.duration_since(oldest);
            let retry_after = self.window.saturating_sub(elapsed);
            let retry_after_ms = u64::try_from(retry_after.as_millis()).unwrap_or(u64::MAX);
            return Err(RpcError::RateLimitExceeded { retry_after_ms });
        }

        entries.push(now);
        Ok(())
    }

    #[must_use]
    pub fn active_key_count(&self) -> usize {
        self.requests.len()
    }

    pub fn prune_expired(&mut self) {
        let now = Instant::now();
        self.requests.retain(|_, entries| {
            entries.retain(|entry| now.duration_since(*entry) <= self.window);
            !entries.is_empty()
        });
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
}
