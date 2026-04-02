//! Governance-facing migration policy helpers for classical -> hybrid -> PQ-only transitions.

use crate::auth::scheme::AuthProfile;
use crate::errors::{AoxcvmError, AoxcvmResult};

/// Ordered migration stages for auth policy evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PqMigrationStage {
    /// Classical signatures are still accepted.
    LegacyCompatible,
    /// Hybrid signatures are permitted and encouraged, but not mandatory.
    HybridCanary,
    /// Hybrid signatures are mandatory for admission.
    HybridRequired,
    /// Only post-quantum signatures are accepted.
    PqStrict,
}

/// Deterministic migration policy snapshot used by governance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PqMigrationPolicy {
    pub stage: PqMigrationStage,
    pub min_pq_signers: usize,
    pub max_classic_signers: usize,
}

impl Default for PqMigrationPolicy {
    fn default() -> Self {
        Self {
            stage: PqMigrationStage::HybridCanary,
            min_pq_signers: 1,
            max_classic_signers: 2,
        }
    }
}

impl PqMigrationPolicy {
    pub const fn auth_profile(self) -> AuthProfile {
        match self.stage {
            PqMigrationStage::LegacyCompatible | PqMigrationStage::HybridCanary => {
                AuthProfile::HybridMandatory
            }
            PqMigrationStage::HybridRequired => AuthProfile::HybridMandatory,
            PqMigrationStage::PqStrict => AuthProfile::PostQuantumStrict,
        }
    }

    /// Validates signer counts against active migration constraints.
    pub fn validate_signer_mix(self, classic_signers: usize, pq_signers: usize) -> AoxcvmResult<()> {
        let total = classic_signers + pq_signers;
        if total == 0 {
            return Err(AoxcvmError::EmptySignatureSet);
        }

        if pq_signers < self.min_pq_signers {
            return Err(AoxcvmError::PolicyViolation(
                "insufficient post-quantum signers for active migration stage",
            ));
        }

        if classic_signers > self.max_classic_signers {
            return Err(AoxcvmError::PolicyViolation(
                "classic signer count exceeds migration policy allowance",
            ));
        }

        match self.stage {
            PqMigrationStage::LegacyCompatible | PqMigrationStage::HybridCanary => Ok(()),
            PqMigrationStage::HybridRequired => {
                if classic_signers == 0 {
                    return Err(AoxcvmError::PolicyViolation(
                        "hybrid-required stage needs at least one classic signer",
                    ));
                }
                Ok(())
            }
            PqMigrationStage::PqStrict => {
                if classic_signers != 0 {
                    return Err(AoxcvmError::PolicyViolation(
                        "pq-strict stage does not allow classic signers",
                    ));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PqMigrationPolicy, PqMigrationStage};

    #[test]
    fn default_policy_accepts_single_classic_plus_pq() {
        let policy = PqMigrationPolicy::default();
        assert!(policy.validate_signer_mix(1, 1).is_ok());
    }

    #[test]
    fn hybrid_required_rejects_missing_classic_signer() {
        let policy = PqMigrationPolicy {
            stage: PqMigrationStage::HybridRequired,
            min_pq_signers: 1,
            max_classic_signers: 2,
        };

        assert!(policy.validate_signer_mix(0, 2).is_err());
    }

    #[test]
    fn pq_strict_rejects_any_classic_signer() {
        let policy = PqMigrationPolicy {
            stage: PqMigrationStage::PqStrict,
            min_pq_signers: 1,
            max_classic_signers: 0,
        };

        assert!(policy.validate_signer_mix(1, 1).is_err());
        assert!(policy.validate_signer_mix(0, 1).is_ok());
    }
}
