//! Authentication envelope structures and deterministic validation rules.

use crate::auth::domains::AuthDomain;
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

    /// Canonical signature witness encoding.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let wire = self.algorithm.wire_id().as_bytes();
        out.extend_from_slice(&(wire.len() as u16).to_be_bytes());
        out.extend_from_slice(wire);
        out.extend_from_slice(&(self.key_id.len() as u16).to_be_bytes());
        out.extend_from_slice(self.key_id.as_bytes());
        out.extend_from_slice(&(self.signature.len() as u32).to_be_bytes());
        out.extend_from_slice(&self.signature);
        out
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
        if AuthDomain::parse(self.domain.as_str()).is_none() {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "domain must be a recognized canonical auth domain",
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

    /// Deterministic witness serialization for signing and hashing.
    pub fn canonical_witness_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.domain.len() as u16).to_be_bytes());
        out.extend_from_slice(self.domain.as_bytes());
        out.extend_from_slice(&self.nonce.to_be_bytes());

        let mut signers = self.signers.clone();
        signers.sort_by(|left, right| {
            left.key_id
                .cmp(&right.key_id)
                .then_with(|| left.algorithm.wire_id().cmp(right.algorithm.wire_id()))
                .then_with(|| left.signature.len().cmp(&right.signature.len()))
        });

        out.extend_from_slice(&(signers.len() as u16).to_be_bytes());
        for signer in &signers {
            let witness = signer.canonical_bytes();
            out.extend_from_slice(&(witness.len() as u32).to_be_bytes());
            out.extend_from_slice(&witness);
        }
        out
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

    #[test]
    fn reject_unknown_domain_identifiers() {
        let envelope = AuthEnvelope {
            domain: "tx-v2-unknown".to_string(),
            nonce: 3,
            signers: vec![signer(SignatureAlgorithm::MlDsa65, "pq-1", 2048)],
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

    #[test]
    fn canonical_serialization_is_order_invariant() {
        let env_a = AuthEnvelope {
            domain: "AOX/TX/V1".to_owned(),
            nonce: 42,
            signers: vec![
                signer(SignatureAlgorithm::MlDsa65, "z-key", 2048),
                signer(SignatureAlgorithm::MlDsa87, "a-key", 3000),
            ],
        };
        let env_b = AuthEnvelope {
            domain: "AOX/TX/V1".to_owned(),
            nonce: 42,
            signers: vec![
                signer(SignatureAlgorithm::MlDsa87, "a-key", 3000),
                signer(SignatureAlgorithm::MlDsa65, "z-key", 2048),
            ],
        };

        assert_eq!(
            env_a.canonical_witness_bytes(),
            env_b.canonical_witness_bytes()
        );
    }
}
