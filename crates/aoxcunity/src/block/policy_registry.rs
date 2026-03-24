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

#[cfg(test)]
mod tests {
    use super::{
        PQ_MANDATORY_START_EPOCH, enforce_signature_policy_migration, resolve_signature_policy,
    };
    use crate::block::types::{BlockBuildError, SignaturePolicy};

    #[test]
    fn resolve_signature_policy_maps_all_supported_values() {
        assert_eq!(
            resolve_signature_policy(1).unwrap(),
            SignaturePolicy::ClassicalOnly
        );
        assert_eq!(
            resolve_signature_policy(2).unwrap(),
            SignaturePolicy::Hybrid
        );
        assert_eq!(
            resolve_signature_policy(3).unwrap(),
            SignaturePolicy::PqPreferred
        );
        assert_eq!(
            resolve_signature_policy(4).unwrap(),
            SignaturePolicy::PqMandatory
        );
    }

    #[test]
    fn resolve_signature_policy_rejects_invalid_values() {
        assert_eq!(
            resolve_signature_policy(0).unwrap_err(),
            BlockBuildError::PostQuantumMissingSignaturePolicy
        );
        assert_eq!(
            resolve_signature_policy(99).unwrap_err(),
            BlockBuildError::PostQuantumInvalidSignaturePolicy
        );
    }

    #[test]
    fn migration_rules_enforce_pq_mandatory_after_cutover() {
        let error = enforce_signature_policy_migration(
            PQ_MANDATORY_START_EPOCH,
            SignaturePolicy::Hybrid,
            true,
        )
        .unwrap_err();
        assert_eq!(error, BlockBuildError::CryptoEpochRequiresPqMandatory);
    }

    #[test]
    fn pq_mandatory_requires_downgrade_protection() {
        let error =
            enforce_signature_policy_migration(1, SignaturePolicy::PqMandatory, false).unwrap_err();
        assert_eq!(
            error,
            BlockBuildError::PqMandatoryRequiresDowngradeProtection
        );
    }
}
