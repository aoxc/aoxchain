// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::MobError;
use sha3::{Digest, Sha3_256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current Unix timestamp in seconds.
pub fn now_epoch_secs() -> Result<u64, MobError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|error| MobError::Time(error.to_string()))
}

/// Returns an uppercase SHA3-256 hex digest for the supplied byte slice.
#[must_use]
pub fn sha3_hex_upper(data: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hex::encode_upper(hasher.finalize())
}

/// Returns a stable uppercase identifier prefix derived from the supplied input.
#[must_use]
pub fn prefixed_id(prefix: &str, parts: &[&[u8]]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(prefix.as_bytes());
    hasher.update([0x00]);
    for part in parts {
        hasher.update(part);
        hasher.update([0x00]);
    }
    let digest = hasher.finalize();
    format!("{}-{}", prefix, hex::encode_upper(&digest[..10]))
}
