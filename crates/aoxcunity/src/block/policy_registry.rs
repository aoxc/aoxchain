use crate::block::types::{BlockBuildError, SignaturePolicy};

/// Canonical policy migration cut-over for PQ mandatory mode.
pub const PQ_MANDATORY_START_EPOCH: u64 = 100;

/// Resolves a signature policy id into a canonical policy enum.
pub fn resolve_signature_policy(policy_id: u32) -> Result<SignaturePolicy, BlockBuildError> {
    match policy_id {
        1 => Ok(SignaturePolicy::ClassicalOnly),
        2 => Ok(SignaturePolicy::Hybrid),
        3 => Ok(SignaturePolicy::PqPreferred),
        4 => Ok(SignaturePolicy::PqMandatory),
        0 => Err(BlockBuildError::PostQuantumMissingSignaturePolicy),
        _ => Err(BlockBuildError::PostQuantumInvalidSignaturePolicy),
    }
}

/// Enforces crypto-epoch migration and downgrade hardening policy.
pub fn enforce_signature_policy_migration(
    crypto_epoch: u64,
    policy: SignaturePolicy,
    downgrade_prohibited: bool,
) -> Result<(), BlockBuildError> {
    if crypto_epoch >= PQ_MANDATORY_START_EPOCH && policy != SignaturePolicy::PqMandatory {
        return Err(BlockBuildError::CryptoEpochRequiresPqMandatory);
    }

    if policy == SignaturePolicy::PqMandatory && !downgrade_prohibited {
        return Err(BlockBuildError::PqMandatoryRequiresDowngradeProtection);
    }

    Ok(())
}
