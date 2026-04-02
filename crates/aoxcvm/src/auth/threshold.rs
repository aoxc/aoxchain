//! Threshold-evaluation helpers for multi-signer auth envelopes.

use crate::auth::envelope::AuthEnvelope;

/// Static threshold policy for envelope signer checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdPolicy {
    pub min_signers: usize,
    pub require_post_quantum: bool,
}

impl ThresholdPolicy {
    /// Evaluates whether an envelope satisfies this threshold policy.
    pub fn is_satisfied_by(self, envelope: &AuthEnvelope) -> bool {
        if envelope.signers.len() < self.min_signers {
            return false;
        }

        if self.require_post_quantum {
            envelope
                .signers
                .iter()
                .any(|entry| entry.algorithm.is_post_quantum())
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
    };

    use super::ThresholdPolicy;

    #[test]
    fn threshold_requires_minimum_signers() {
        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa65,
                key_id: "pq-a".to_owned(),
                signature: vec![1_u8; 128],
            }],
        };

        let policy = ThresholdPolicy {
            min_signers: 2,
            require_post_quantum: true,
        };

        assert!(!policy.is_satisfied_by(&envelope));
    }

    #[test]
    fn threshold_can_require_pq_presence() {
        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 2,
            signers: vec![
                SignatureEntry {
                    algorithm: SignatureAlgorithm::Ed25519,
                    key_id: "classic".to_owned(),
                    signature: vec![2_u8; 64],
                },
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa87,
                    key_id: "pq".to_owned(),
                    signature: vec![3_u8; 192],
                },
            ],
        };

        let policy = ThresholdPolicy {
            min_signers: 2,
            require_post_quantum: true,
        };

        assert!(policy.is_satisfied_by(&envelope));
    }
}
