use std::time::Duration;

/// Global chain runtime configuration.
///
/// This structure defines timing parameters that directly influence
/// consensus scheduling and block production cadence.
///
/// Security considerations:
/// - Extremely small block intervals can lead to network instability
///   and excessive fork rates.
/// - Extremely large intervals degrade transaction latency and
///   reduce network liveness.
///
/// These parameters must therefore be validated during node bootstrap.
#[derive(Debug, Clone, Copy)]
pub struct ChainConfig {
    /// Target block production interval in seconds.
    ///
    /// This value determines how frequently the proposer layer attempts
    /// to construct a new candidate block.
    pub block_time_secs: u64,
}

impl ChainConfig {
    /// Validates configuration values for safe runtime operation.
    ///
    /// Returns an error if any parameter falls outside the accepted range.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.block_time_secs == 0 {
            return Err("block_time_secs must be greater than zero");
        }

        if self.block_time_secs < 2 {
            return Err("block_time_secs is too small and may destabilize consensus");
        }

        if self.block_time_secs > 600 {
            return Err("block_time_secs exceeds reasonable upper bound");
        }

        Ok(())
    }

    /// Returns the block interval as a `Duration`.
    pub fn block_interval(&self) -> Duration {
        Duration::from_secs(self.block_time_secs)
    }
}

impl Default for ChainConfig {
    /// Returns the canonical AOXC default configuration.
    ///
    /// Current policy:
    /// - 6 second block interval provides a balance between
    ///   confirmation latency and network stability.
    fn default() -> Self {
        Self {
            block_time_secs: 6,
        }
    }
}

