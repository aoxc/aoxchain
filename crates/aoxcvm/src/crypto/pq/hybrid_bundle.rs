//! Deterministic helpers for hybrid classical + post-quantum signature bundles.
//!
//! This module does not verify cryptography; it validates bounded metadata for
//! governance/policy checks and deterministic transaction admission.

use crate::auth::scheme::SignatureAlgorithm;
use crate::errors::{AoxcvmError, AoxcvmResult};

/// Descriptor for a single signer witness carried in a hybrid bundle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridSignerWitness {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub signature_len: usize,
}

impl HybridSignerWitness {
    pub fn is_post_quantum(&self) -> bool {
        self.algorithm.is_post_quantum()
    }

    pub fn encoded_size_hint(&self) -> usize {
        self.key_id.len() + self.algorithm.wire_id().len() + self.signature_len
    }
}

/// Deterministic container used by policy layers that require mixed classical
/// and post-quantum authorization.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HybridSignatureBundle {
    pub signers: Vec<HybridSignerWitness>,
}

impl HybridSignatureBundle {
    /// Validates bounded structure and mandatory hybrid composition.
    pub fn validate(&self, limits: HybridBundleLimits) -> AoxcvmResult<()> {
        if self.signers.is_empty() {
            return Err(AoxcvmError::EmptySignatureSet);
        }
        if self.signers.len() > limits.max_signatures {
            return Err(AoxcvmError::AuthLimitExceeded {
                limit: "max_signatures",
                got: self.signers.len(),
                max: limits.max_signatures,
            });
        }

        let mut has_classic = false;
        let mut has_pq = false;

        for signer in &self.signers {
            if signer.key_id.is_empty() {
                return Err(AoxcvmError::InvalidSignatureMetadata(
                    "key_id must not be empty",
                ));
            }
            if signer.signature_len == 0 {
                return Err(AoxcvmError::InvalidSignatureMetadata(
                    "signature_len must be non-zero",
                ));
            }
            if signer.signature_len > limits.max_signature_bytes {
                return Err(AoxcvmError::AuthLimitExceeded {
                    limit: "max_signature_bytes",
                    got: signer.signature_len,
                    max: limits.max_signature_bytes,
                });
            }
            if signer.is_post_quantum() {
                has_pq = true;
            } else {
                has_classic = true;
            }
        }

        if !has_classic || !has_pq {
            return Err(AoxcvmError::PolicyViolation(
                "hybrid bundle requires at least one classic and one post-quantum signer",
            ));
        }

        Ok(())
    }

    /// Deterministic aggregate byte-length hint used by gas/metering layers.
    pub fn encoded_size_hint(&self) -> usize {
        self.signers
            .iter()
            .map(HybridSignerWitness::encoded_size_hint)
            .sum()
    }
}

/// Fixed admission limits for deterministic hybrid bundle validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HybridBundleLimits {
    pub max_signatures: usize,
    pub max_signature_bytes: usize,
}

impl Default for HybridBundleLimits {
    fn default() -> Self {
        Self {
            max_signatures: 16,
            max_signature_bytes: 8192,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HybridBundleLimits, HybridSignatureBundle, HybridSignerWitness};
    use crate::auth::scheme::SignatureAlgorithm;

    fn witness(algorithm: SignatureAlgorithm, key_id: &str, signature_len: usize) -> HybridSignerWitness {
        HybridSignerWitness {
            algorithm,
            key_id: key_id.to_owned(),
            signature_len,
        }
    }

    #[test]
    fn accepts_hybrid_classic_and_pq_signers() {
        let bundle = HybridSignatureBundle {
            signers: vec![
                witness(SignatureAlgorithm::Ed25519, "classic-1", 64),
                witness(SignatureAlgorithm::MlDsa65, "pq-1", 2304),
            ],
        };

        assert!(bundle.validate(HybridBundleLimits::default()).is_ok());
    }

    #[test]
    fn rejects_bundle_without_classic_signer() {
        let bundle = HybridSignatureBundle {
            signers: vec![witness(SignatureAlgorithm::MlDsa87, "pq-1", 3309)],
        };

        assert!(bundle.validate(HybridBundleLimits::default()).is_err());
    }

    #[test]
    fn encoded_size_hint_sums_metadata_and_signatures() {
        let bundle = HybridSignatureBundle {
            signers: vec![
                witness(SignatureAlgorithm::Ed25519, "classic-key", 64),
                witness(SignatureAlgorithm::MlDsa65, "pq-key", 2048),
            ],
        };

        let expected = "classic-key".len()
            + SignatureAlgorithm::Ed25519.wire_id().len()
            + 64
            + "pq-key".len()
            + SignatureAlgorithm::MlDsa65.wire_id().len()
            + 2048;

        assert_eq!(bundle.encoded_size_hint(), expected);
    }
}
