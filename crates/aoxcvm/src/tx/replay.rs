//! Replay-prevention helpers.

/// Per-sender nonce tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NonceWindow {
    last_seen: Option<u64>,
}

impl NonceWindow {
    pub fn check_and_update(&mut self, nonce: u64) -> bool {
        match self.last_seen {
            Some(last) if nonce <= last => false,
            _ => {
                self.last_seen = Some(nonce);
                true
            }
        }
    }
}
