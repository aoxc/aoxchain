//! Authentication envelope structures and deterministic validation rules.

use crate::auth::scheme::{AuthProfile, SignatureAlgorithm};
use crate::errors::{AoxcvmError, AoxcvmResult};

/// Single signature witness attached to an auth envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureEntry {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub signature: Vec<u8>,
}

impl SignatureEntry {
    pub fn encoded_size(&self) -> usize {
        self.key_id.len() + self.signature.len() + self.algorithm.wire_id().len()
    }
}

/// Transaction auth envelope for policy and replay validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthEnvelope {
    pub domain: String,
    pub nonce: u64,
    pub signers: Vec<SignatureEntry>,
}

impl AuthEnvelope {
    pub fn validate(&self, profile: AuthProfile, limits: AuthEnvelopeLimits) -> AoxcvmResult<()> {
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
        if self.domain.is_empty() {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "domain must not be empty",
            ));
        }

        let mut algorithms = Vec::with_capacity(self.signers.len());
        for signer in &self.signers {
            if signer.key_id.is_empty() {
                return Err(AoxcvmError::InvalidSignatureMetadata(
                    "key_id must not be empty",
                ));
            }
            if signer.signature.is_empty() {
                return Err(AoxcvmError::InvalidSignatureMetadata(
                    "signature must not be empty",
                ));
            }
            if signer.signature.len() > limits.max_signature_bytes {
                return Err(AoxcvmError::AuthLimitExceeded {
                    limit: "max_signature_bytes",
                    got: signer.signature.len(),
                    max: limits.max_signature_bytes,
                });
            }
            algorithms.push(signer.algorithm);
        }

        if !profile.signer_set_is_valid(&algorithms) {
            return Err(AoxcvmError::PolicyViolation(
                "signer set does not satisfy active auth profile",
            ));
        }

        Ok(())
    }
}

/// Bounded limits for deterministic envelope validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthEnvelopeLimits {
    pub max_signatures: usize,
    pub max_signature_bytes: usize,
}

impl Default for AuthEnvelopeLimits {
    fn default() -> Self {
        Self {
            max_signatures: 16,
            max_signature_bytes: 4096,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::scheme::SignatureAlgorithm;

    use super::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry};
    use crate::auth::scheme::AuthProfile;

    fn signer(algorithm: SignatureAlgorithm, key_id: &str, bytes: usize) -> SignatureEntry {
        SignatureEntry {
            algorithm,
            key_id: key_id.to_owned(),
            signature: vec![7_u8; bytes],
        }
    }

    #[test]
    fn hybrid_profile_accepts_mixed_signers() {
        let envelope = AuthEnvelope {
            domain: "tx".to_string(),
            nonce: 1,
            signers: vec![
                signer(SignatureAlgorithm::Ed25519, "classic-1", 64),
                signer(SignatureAlgorithm::MlDsa65, "pq-1", 2048),
            ],
        };
        assert!(
            envelope
                .validate(AuthProfile::HybridMandatory, AuthEnvelopeLimits::default())
                .is_ok()
        );
    }

    #[test]
    fn strict_pq_rejects_classic_signatures() {
        let envelope = AuthEnvelope {
            domain: "tx".to_string(),
            nonce: 2,
            signers: vec![
                signer(SignatureAlgorithm::MlDsa87, "pq-1", 3000),
                signer(SignatureAlgorithm::Ed25519, "classic-1", 64),
            ],
        };
        assert!(
            envelope
                .validate(
                    AuthProfile::PostQuantumStrict,
                    AuthEnvelopeLimits::default()
                )
                .is_err()
        );
    }
}
