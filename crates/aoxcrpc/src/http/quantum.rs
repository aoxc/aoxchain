// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::types::{QuantumCryptoProfile, QuantumHashLevel, QuantumKeyLevel};

/// Returns a baseline post-quantum cryptography profile for RPC clients.
///
/// This profile is intentionally simple so operators can expose it as a
/// machine-readable "what algorithms are expected" endpoint payload.
#[must_use]
pub fn quantum_crypto_profile() -> QuantumCryptoProfile {
    QuantumCryptoProfile {
        profile_version: "v1".to_string(),
        assurance_target_percent: 99.9999,
        hash_levels: vec![
            QuantumHashLevel {
                algorithm: "SHA3-512".to_string(),
                security_bits_classical: 256,
                security_bits_quantum_estimated: 256,
                purpose: "transaction and state commitment hashing".to_string(),
            },
            QuantumHashLevel {
                algorithm: "Argon2id".to_string(),
                security_bits_classical: 256,
                security_bits_quantum_estimated: 128,
                purpose: "password and keyfile KDF hardening".to_string(),
            },
        ],
        key_levels: vec![
            QuantumKeyLevel {
                primitive: "ML-KEM-768 + X25519 (hybrid)".to_string(),
                security_bits_classical: 192,
                security_bits_quantum_estimated: 192,
                purpose: "session key establishment with migration safety".to_string(),
            },
            QuantumKeyLevel {
                primitive: "ML-DSA-65".to_string(),
                security_bits_classical: 192,
                security_bits_quantum_estimated: 192,
                purpose: "node and transaction signature policy".to_string(),
            },
        ],
        notes: vec![
            "Target expresses operational confidence, not absolute unbreakability".to_string(),
            "Profile must be combined with rotation, audit, and secure implementation controls"
                .to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantum_profile_contains_expected_baseline_algorithms() {
        let profile = quantum_crypto_profile();

        assert_eq!(profile.profile_version, "v1");
        assert_eq!(profile.assurance_target_percent, 99.9999);
        assert!(
            profile
                .hash_levels
                .iter()
                .any(|entry| entry.algorithm == "SHA3-512")
        );
        assert!(
            profile
                .key_levels
                .iter()
                .any(|entry| entry.primitive.contains("ML-KEM-768"))
        );
    }
}
