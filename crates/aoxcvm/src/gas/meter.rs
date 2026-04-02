//! Deterministic gas metering primitives for AOXCVM phase-1 execution.

/// Error returned by [`GasMeter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GasError {
    /// The call would exceed the configured gas limit.
    OutOfGas,
}

/// Deterministic gas meter with monotonic usage accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GasMeter {
    limit: u64,
    used: u64,
}

impl GasMeter {
    /// Creates a new gas meter.
    pub const fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }

    /// Returns the configured gas limit.
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns gas already consumed.
    pub const fn used(&self) -> u64 {
        self.used
    }

    /// Returns gas available for future operations.
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Charges gas and fails deterministically if there is not enough remaining budget.
    pub fn charge(&mut self, amount: u64) -> Result<(), GasError> {
        let next = self.used.checked_add(amount).ok_or(GasError::OutOfGas)?;
        if next > self.limit {
            return Err(GasError::OutOfGas);
        }
        self.used = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{GasError, GasMeter};

    #[test]
    fn charge_tracks_usage() {
        let mut meter = GasMeter::new(10);
        meter.charge(4).expect("within budget");
        assert_eq!(meter.used(), 4);
        assert_eq!(meter.remaining(), 6);
    }

    #[test]
    fn out_of_gas_rejected() {
        let mut meter = GasMeter::new(5);
        assert_eq!(meter.charge(6), Err(GasError::OutOfGas));
        assert_eq!(meter.used(), 0);
    }
}
