//! Post-quantum-strict authentication policy helpers.

use crate::auth::envelope::{AuthEnvelope, AuthEnvelopeLimits};
use crate::auth::scheme::AuthProfile;
use crate::errors::AoxcvmResult;

/// Runtime baseline for post-quantum-only auth deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct QuantumPolicy {
    /// Hard limits applied before signature verification.
    pub limits: AuthEnvelopeLimits,
}

impl QuantumPolicy {
    /// Validate that an envelope satisfies the mandatory PQ-only profile.
    pub fn validate(&self, envelope: &AuthEnvelope) -> AoxcvmResult<()> {
        envelope.validate(AuthProfile::PostQuantumStrict, self.limits)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
    };

    use super::QuantumPolicy;

    #[test]
    fn quantum_policy_rejects_classical_signers() {
        let env = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 7,
            signers: vec![
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa87,
                    key_id: "pq".to_owned(),
                    signature: vec![1_u8; 2500],
                },
                SignatureEntry {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "classic".to_owned(),
                    signature: vec![2_u8; 64],
                },
            ],
        };
        assert!(QuantumPolicy::default().validate(&env).is_err());
    }

    #[test]
    fn quantum_policy_accepts_only_pq_signers() {
        let env = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 8,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa65,
                key_id: "pq-only".to_owned(),
                signature: vec![3_u8; 2500],
            }],
        };
        assert!(QuantumPolicy::default().validate(&env).is_ok());
    }
}
