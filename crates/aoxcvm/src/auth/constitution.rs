//! Cryptographic constitution baseline for quantum-first AOXCVM deployments.

use crate::{
    auth::{
        domains::AuthDomain,
        envelope::AuthEnvelope,
        recovery::ConstitutionalRecoveryPolicy,
        scheme::{AuthProfile, SignatureAlgorithm},
    },
    errors::{AoxcvmError, AoxcvmResult},
};

/// Address derivation policy controlled by the constitutional profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressDerivationModel {
    /// Address is bound to policy/key commitments of PQ smart accounts.
    PqSmartAccountCommitmentV1,
}

/// Validator identity policy format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorIdentityFormat {
    /// Separate stake identity and online consensus keys.
    StakeAndConsensusKeysV1,
}

/// Account validation policy surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountValidationPolicy {
    /// Native accounts must be quantum smart accounts.
    PqSmartAccountsOnly,
}

/// Key rotation rules enforced by constitution-aware checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyRotationRules {
    /// Rotation must preserve post-quantum continuity across generations.
    pub require_pq_continuity: bool,
    /// Rotation must preserve at least one algorithm overlap.
    pub require_algorithm_overlap: bool,
}

/// Baseline cryptographic constitution for chain operation and recovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CryptographicConstitution {
    /// Primary operational signature scheme.
    pub primary_signature_scheme: SignatureAlgorithm,
    /// Constitutional-recovery signature scheme.
    pub constitutional_recovery_scheme: SignatureAlgorithm,
    /// Native account validation posture.
    pub account_validation_policy: AccountValidationPolicy,
    /// Address derivation format.
    pub address_derivation_model: AddressDerivationModel,
    /// Validator identity format.
    pub validator_identity_format: ValidatorIdentityFormat,
    /// Key-rotation constraints.
    pub key_rotation_rules: KeyRotationRules,
}

impl CryptographicConstitution {
    /// Validates an operational envelope against constitution baseline.
    pub fn validate_operational_envelope(&self, envelope: &AuthEnvelope) -> AoxcvmResult<()> {
        let domain = AuthDomain::parse(envelope.domain.as_str())
            .ok_or(AoxcvmError::InvalidSignatureMetadata("unknown auth domain"))?;

        if domain == AuthDomain::ConstitutionalRecovery {
            return Err(AoxcvmError::PolicyViolation(
                "operational lane cannot use constitutional-recovery domain",
            ));
        }

        envelope.validate(AuthProfile::PostQuantumStrict, Default::default())?;

        if envelope
            .signers
            .iter()
            .any(|entry| entry.algorithm.is_constitutional_recovery())
        {
            return Err(AoxcvmError::PolicyViolation(
                "operational lane cannot use constitutional recovery signatures",
            ));
        }

        if !envelope
            .signers
            .iter()
            .any(|entry| entry.algorithm == self.primary_signature_scheme)
        {
            return Err(AoxcvmError::PolicyViolation(
                "operational envelope must include primary signature scheme",
            ));
        }

        Ok(())
    }

    /// Validates a constitutional-recovery envelope against constitution baseline.
    pub fn validate_constitutional_recovery_envelope(
        &self,
        envelope: &AuthEnvelope,
    ) -> AoxcvmResult<()> {
        ConstitutionalRecoveryPolicy::default().validate(envelope)?;
        if envelope
            .signers
            .iter()
            .any(|entry| entry.algorithm != self.constitutional_recovery_scheme)
        {
            return Err(AoxcvmError::PolicyViolation(
                "constitutional envelope uses non-constitutional algorithm",
            ));
        }
        Ok(())
    }
}

impl Default for CryptographicConstitution {
    fn default() -> Self {
        Self {
            primary_signature_scheme: SignatureAlgorithm::MlDsa65,
            constitutional_recovery_scheme: SignatureAlgorithm::SlhDsa128s,
            account_validation_policy: AccountValidationPolicy::PqSmartAccountsOnly,
            address_derivation_model: AddressDerivationModel::PqSmartAccountCommitmentV1,
            validator_identity_format: ValidatorIdentityFormat::StakeAndConsensusKeysV1,
            key_rotation_rules: KeyRotationRules {
                require_pq_continuity: true,
                require_algorithm_overlap: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        constitution::CryptographicConstitution,
        domains::AuthDomain,
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
        test_fixtures::fixture_signature_len,
    };

    #[test]
    fn operational_envelope_requires_primary_ml_dsa() {
        let constitution = CryptographicConstitution::default();
        let envelope = AuthEnvelope {
            domain: AuthDomain::Transaction.canonical_tag().to_owned(),
            nonce: 1,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa87,
                key_id: "pq-1".to_owned(),
                signature: vec![7_u8; 3200],
            }],
        };

        assert!(
            constitution
                .validate_operational_envelope(&envelope)
                .is_err()
        );
    }

    #[test]
    fn constitutional_recovery_requires_slh_dsa() {
        let constitution = CryptographicConstitution::default();
        let envelope = AuthEnvelope {
            domain: AuthDomain::ConstitutionalRecovery
                .canonical_tag()
                .to_owned(),
            nonce: 9,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::SlhDsa128s,
                key_id: "constitution-1".to_owned(),
                signature: vec![8_u8; fixture_signature_len(SignatureAlgorithm::SlhDsa128s)],
            }],
        };

        assert!(
            constitution
                .validate_constitutional_recovery_envelope(&envelope)
                .is_ok()
        );
    }
}
