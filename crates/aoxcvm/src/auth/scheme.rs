//! Signature-scheme registry primitives for AOXCVM authentication.
//!
//! The goal of this module is to provide a deterministic, serializable
//! representation of accepted signature suites so policy and envelope layers
//! can evolve toward crypto-agile and post-quantum-safe verification.

/// Curve and post-quantum families currently recognized by AOXCVM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignatureAlgorithm {
    /// Classic EdDSA over Curve25519.
    Ed25519,
    /// NIST P-256 ECDSA (legacy interoperability profile).
    EcdsaP256,
    /// ML-DSA-65 (formerly Dilithium2-level profile).
    MlDsa65,
    /// ML-DSA-87 (higher-security post-quantum profile).
    MlDsa87,
    /// SLH-DSA-128s (SPHINCS+ family) for constitutional recovery actions.
    SlhDsa128s,
}

impl SignatureAlgorithm {
    /// Stable wire identifier for governance snapshots and transaction metadata.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::Ed25519 => "ed25519",
            Self::EcdsaP256 => "ecdsa-p256",
            Self::MlDsa65 => "ml-dsa-65",
            Self::MlDsa87 => "ml-dsa-87",
            Self::SlhDsa128s => "slh-dsa-128s",
        }
    }

    /// Whether the algorithm is considered post-quantum secure.
    pub const fn is_post_quantum(self) -> bool {
        matches!(self, Self::MlDsa65 | Self::MlDsa87 | Self::SlhDsa128s)
    }

    /// Whether the algorithm is reserved for constitutional recovery operations.
    pub const fn is_constitutional_recovery(self) -> bool {
        matches!(self, Self::SlhDsa128s)
    }
}

/// Deterministic policy profile for signer verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuthProfile {
    /// Classic-only mode for legacy integration testing.
    Legacy,
    /// Requires at least one classical and one PQ signature.
    HybridMandatory,
    /// Post-quantum-only mode.
    #[default]
    PostQuantumStrict,
}

impl AuthProfile {
    /// Returns whether an algorithm can participate in this profile.
    pub const fn allows(self, algorithm: SignatureAlgorithm) -> bool {
        match self {
            Self::Legacy => matches!(
                algorithm,
                SignatureAlgorithm::Ed25519 | SignatureAlgorithm::EcdsaP256
            ),
            Self::HybridMandatory => true,
            Self::PostQuantumStrict => algorithm.is_post_quantum(),
        }
    }

    /// Checks if a signer set satisfies profile requirements.
    pub fn signer_set_is_valid(self, algorithms: &[SignatureAlgorithm]) -> bool {
        if algorithms.is_empty() {
            return false;
        }

        let has_classic = algorithms.iter().any(|a| !a.is_post_quantum());
        let has_pq = algorithms.iter().any(|a| a.is_post_quantum());
        let all_allowed = algorithms.iter().all(|a| self.allows(*a));

        match self {
            Self::Legacy => all_allowed && has_classic && !has_pq,
            Self::HybridMandatory => all_allowed && has_classic && has_pq,
            Self::PostQuantumStrict => all_allowed && has_pq,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthProfile, SignatureAlgorithm};

    #[test]
    fn wire_ids_are_stable() {
        assert_eq!(SignatureAlgorithm::Ed25519.wire_id(), "ed25519");
        assert_eq!(SignatureAlgorithm::EcdsaP256.wire_id(), "ecdsa-p256");
        assert_eq!(SignatureAlgorithm::MlDsa65.wire_id(), "ml-dsa-65");
        assert_eq!(SignatureAlgorithm::MlDsa87.wire_id(), "ml-dsa-87");
        assert_eq!(SignatureAlgorithm::SlhDsa128s.wire_id(), "slh-dsa-128s");
    }

    #[test]
    fn hybrid_profile_requires_classic_and_pq() {
        let hybrid = AuthProfile::HybridMandatory;
        assert!(
            hybrid.signer_set_is_valid(&[SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65])
        );
        assert!(!hybrid.signer_set_is_valid(&[SignatureAlgorithm::MlDsa65]));
        assert!(!hybrid.signer_set_is_valid(&[SignatureAlgorithm::Ed25519]));
    }

    #[test]
    fn post_quantum_profile_blocks_classic_keys() {
        let pq_only = AuthProfile::PostQuantumStrict;
        assert!(
            pq_only
                .signer_set_is_valid(&[SignatureAlgorithm::MlDsa65, SignatureAlgorithm::MlDsa87])
        );
        assert!(
            !pq_only
                .signer_set_is_valid(&[SignatureAlgorithm::Ed25519, SignatureAlgorithm::MlDsa65])
        );
    }
}
