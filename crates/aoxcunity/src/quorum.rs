use crate::error::ConsensusError;

/// Quorum threshold expressed in numerator/denominator form.
///
/// This avoids floating-point arithmetic in a consensus-adjacent context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuorumThreshold {
    pub numerator: u64,
    pub denominator: u64,
}

impl QuorumThreshold {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, ConsensusError> {
        if denominator == 0 || numerator == 0 || numerator > denominator {
            return Err(ConsensusError::InvalidQuorumThreshold);
        }

        Ok(Self {
            numerator,
            denominator,
        })
    }

    pub fn two_thirds() -> Self {
        Self {
            numerator: 2,
            denominator: 3,
        }
    }

    pub fn is_reached(&self, observed_power: u64, total_power: u64) -> bool {
        if total_power == 0 {
            return false;
        }

        observed_power.saturating_mul(self.denominator)
            >= total_power.saturating_mul(self.numerator)
    }
}
