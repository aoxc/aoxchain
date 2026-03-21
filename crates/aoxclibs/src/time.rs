use crate::types::LibError;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_unix_timestamp() -> Result<u64, LibError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| LibError::TimeError("system clock before epoch".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_validity() {
        let ts = current_unix_timestamp().expect("Time failed");
        // 2026 yılındayız, timestamp 1.7 milyardan büyük olmalı
        assert!(ts > 1700000000);
    }
}
