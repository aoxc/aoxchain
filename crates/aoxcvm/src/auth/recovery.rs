//! Constitutional-recovery policy for emergency governance controls.

use crate::{
    auth::{
        domains::AuthDomain,
        envelope::{AuthEnvelope, AuthEnvelopeLimits},
        scheme::{AuthProfile, SignatureAlgorithm},
    },
    errors::{AoxcvmError, AoxcvmResult},
};

/// Hard policy guard for constitutional-recovery transactions.
///
/// Recovery envelopes must:
/// - use the constitutional-recovery domain,
/// - satisfy post-quantum strict profile checks,
/// - contain only SLH-DSA signatures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConstitutionalRecoveryPolicy {
    /// Deterministic limits for envelope shape.
    pub limits: AuthEnvelopeLimits,
}

impl ConstitutionalRecoveryPolicy {
    /// Validates recovery envelope invariants.
    pub fn validate(self, envelope: &AuthEnvelope) -> AoxcvmResult<()> {
        let parsed_domain = AuthDomain::parse(envelope.domain.as_str()).ok_or(
            AoxcvmError::InvalidSignatureMetadata(
                "constitutional recovery requires a recognized auth domain",
            ),
        )?;
        if parsed_domain != AuthDomain::ConstitutionalRecovery {
            return Err(AoxcvmError::PolicyViolation(
                "constitutional recovery requires constitutional-recovery domain",
            ));
        }

        envelope.validate(AuthProfile::PostQuantumStrict, self.limits)?;

        if envelope
            .signers
            .iter()
            .any(|entry| !matches!(entry.algorithm, SignatureAlgorithm::SlhDsa128s))
        {
            return Err(AoxcvmError::PolicyViolation(
                "constitutional recovery requires SLH-DSA-only witnesses",
            ));
        }

        Ok(())
    }
}

impl Default for ConstitutionalRecoveryPolicy {
    fn default() -> Self {
        Self {
            limits: AuthEnvelopeLimits {
                max_signatures: 32,
                max_signature_bytes: 16384,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        domains::AuthDomain,
        envelope::{AuthEnvelope, SignatureEntry},
        recovery::ConstitutionalRecoveryPolicy,
        scheme::SignatureAlgorithm,
        test_fixtures::fixture_signature_len,
    };

    #[test]
    fn recovery_accepts_constitutional_slh_only_payload() {
        let envelope = AuthEnvelope {
            domain: AuthDomain::ConstitutionalRecovery
                .canonical_tag()
                .to_owned(),
            nonce: 9,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::SlhDsa128s,
                key_id: "constitution-1".to_owned(),
                signature: vec![4_u8; 3000],
            }],
        };
        assert!(
            ConstitutionalRecoveryPolicy::default()
                .validate(&envelope)
                .is_ok()
        );
    }

    #[test]
    fn recovery_rejects_non_recovery_algorithm() {
        let envelope = AuthEnvelope {
            domain: AuthDomain::ConstitutionalRecovery
                .canonical_tag()
                .to_owned(),
            nonce: 10,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa87,
                key_id: "constitution-1".to_owned(),
                signature: vec![5_u8; 3309],
            }],
        };
        assert!(
            ConstitutionalRecoveryPolicy::default()
                .validate(&envelope)
                .is_err()
        );
    }
}
