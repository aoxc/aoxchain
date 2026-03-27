use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::LibError;

/// Returns the current UNIX timestamp in whole seconds.
///
/// The value represents the number of elapsed seconds since
/// `1970-01-01T00:00:00Z` (UNIX epoch).
///
/// # Errors
///
/// Returns [`LibError::TimeError`] if the current system clock is earlier than
/// the UNIX epoch.
pub fn current_unix_timestamp() -> Result<u64, LibError> {
    unix_timestamp_from_system_time(SystemTime::now())
}

/// Returns the current UNIX timestamp in whole milliseconds.
///
/// The value represents the number of elapsed milliseconds since
/// `1970-01-01T00:00:00Z` (UNIX epoch).
///
/// # Errors
///
/// Returns [`LibError::TimeError`] if the current system clock is earlier than
/// the UNIX epoch.
pub fn current_unix_timestamp_millis() -> Result<u128, LibError> {
    unix_timestamp_millis_from_system_time(SystemTime::now())
}

/// Converts a [`SystemTime`] instance into a UNIX timestamp in whole seconds.
///
/// This function is deterministic for the supplied input and is preferable in
/// tests or library code that should not depend directly on ambient wall-clock
/// access.
///
/// # Errors
///
/// Returns [`LibError::TimeError`] if `time` is earlier than the UNIX epoch.
pub fn unix_timestamp_from_system_time(time: SystemTime) -> Result<u64, LibError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| LibError::TimeError("system time is earlier than UNIX epoch".to_owned()))
}

/// Converts a [`SystemTime`] instance into a UNIX timestamp in whole milliseconds.
///
/// This function preserves millisecond precision exposed by the standard library
/// duration conversion.
///
/// # Errors
///
/// Returns [`LibError::TimeError`] if `time` is earlier than the UNIX epoch.
pub fn unix_timestamp_millis_from_system_time(time: SystemTime) -> Result<u128, LibError> {
    time.duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|_| LibError::TimeError("system time is earlier than UNIX epoch".to_owned()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_current_unix_timestamp_is_reasonable() {
        let timestamp =
            current_unix_timestamp().expect("current UNIX timestamp retrieval must succeed");

        assert!(timestamp > 1_700_000_000);
    }

    #[test]
    fn test_current_unix_timestamp_millis_is_reasonable() {
        let timestamp_millis = current_unix_timestamp_millis()
            .expect("current UNIX millisecond timestamp retrieval must succeed");

        assert!(timestamp_millis > 1_700_000_000_000);
    }

    #[test]
    fn test_unix_timestamp_from_system_time_is_deterministic() {
        let fixed_time = UNIX_EPOCH + Duration::from_secs(42);

        let timestamp =
            unix_timestamp_from_system_time(fixed_time).expect("fixed second conversion must succeed");

        assert_eq!(timestamp, 42);
    }

    #[test]
    fn test_unix_timestamp_millis_from_system_time_is_deterministic() {
        let fixed_time = UNIX_EPOCH + Duration::from_millis(42_123);

        let timestamp_millis = unix_timestamp_millis_from_system_time(fixed_time)
            .expect("fixed millisecond conversion must succeed");

        assert_eq!(timestamp_millis, 42_123);
    }

    #[test]
    fn test_unix_timestamp_from_system_time_rejects_pre_epoch_input() {
        let pre_epoch_time = UNIX_EPOCH - Duration::from_secs(1);

        let error = unix_timestamp_from_system_time(pre_epoch_time)
            .expect_err("pre-epoch second conversion must fail");

        assert_eq!(
            error,
            LibError::TimeError("system time is earlier than UNIX epoch".to_owned())
        );
    }

    #[test]
    fn test_unix_timestamp_millis_from_system_time_rejects_pre_epoch_input() {
        let pre_epoch_time = UNIX_EPOCH - Duration::from_millis(1);

        let error = unix_timestamp_millis_from_system_time(pre_epoch_time)
            .expect_err("pre-epoch millisecond conversion must fail");

        assert_eq!(
            error,
            LibError::TimeError("system time is earlier than UNIX epoch".to_owned())
        );
    }
}
