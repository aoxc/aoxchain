//! Hybrid classical+PQ authentication policy helpers.

use crate::auth::envelope::{AuthEnvelope, AuthEnvelopeLimits};
use crate::auth::scheme::AuthProfile;
use crate::errors::AoxcvmResult;

/// Runtime baseline for hybrid-auth deployments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HybridPolicy {
    /// Hard limits applied before signature verification.
    pub limits: AuthEnvelopeLimits,
}

impl Default for HybridPolicy {
    fn default() -> Self {
        Self {
            limits: AuthEnvelopeLimits::default(),
        }
    }
}

impl HybridPolicy {
    /// Validate that an envelope satisfies the mandatory hybrid profile.
    pub fn validate(&self, envelope: &AuthEnvelope) -> AoxcvmResult<()> {
        envelope.validate(AuthProfile::HybridMandatory, self.limits)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
    };

    use super::HybridPolicy;

    #[test]
    fn hybrid_policy_rejects_single_pq_signer() {
        let env = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 7,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa65,
                key_id: "pq-only".to_owned(),
                signature: vec![1_u8; 2500],
            }],
        };
        assert!(HybridPolicy::default().validate(&env).is_err());
    }
}
