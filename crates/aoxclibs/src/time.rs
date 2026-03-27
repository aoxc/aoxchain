use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::LibError;

use crate::types::LibError;

pub fn current_unix_timestamp() -> Result<u64, LibError> {
    unix_timestamp_from_system_time(SystemTime::now())
}

pub fn current_unix_timestamp_millis() -> Result<u128, LibError> {
    unix_timestamp_millis_from_system_time(SystemTime::now())
}

pub fn unix_timestamp_from_system_time(time: SystemTime) -> Result<u64, LibError> {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| LibError::TimeError("system clock before epoch".to_owned()))
}

pub fn unix_timestamp_millis_from_system_time(time: SystemTime) -> Result<u128, LibError> {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .map_err(|_| LibError::TimeError("system clock before epoch".to_owned()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_timestamp_validity() {
        let ts = current_unix_timestamp().expect("Time failed");
        // 2026 yılındayız, timestamp 1.7 milyardan büyük olmalı
        assert!(ts > 1_700_000_000);
    }

    #[test]
    fn test_timestamp_millis_validity() {
        let ts_ms = current_unix_timestamp_millis().expect("Time millis failed");
        assert!(ts_ms > 1_700_000_000_000);
    }

    #[test]
    fn test_deterministic_system_time_conversion() {
        let fixed = UNIX_EPOCH + Duration::from_secs(42);
        let ts = unix_timestamp_from_system_time(fixed).expect("fixed seconds conversion failed");
        assert_eq!(ts, 42);

        let fixed_ms = UNIX_EPOCH + Duration::from_millis(42_123);
        let ts_ms = unix_timestamp_millis_from_system_time(fixed_ms)
            .expect("fixed milliseconds conversion failed");
        assert_eq!(ts_ms, 42_123);
    }
}
