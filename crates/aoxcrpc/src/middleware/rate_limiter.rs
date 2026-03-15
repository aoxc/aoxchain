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
            return Err(RpcError::RateLimitExceeded);
        }

        entries.push(now);
        Ok(())
    }
}
