// AOXC MIT License
// Quantum-native kernel policy profile.
//
// This module provides a deterministic policy surface for configuring
// cryptographic primitives that must be available at protocol-kernel scope.
//
// Design intent:
// - default to post-quantum safe primitives,
// - reject legacy-only profiles,
// - remain crypto-agile through explicit versioned profile fields.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Signature schemes supported by AOXC kernel policy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SignatureScheme {
    MlDsa65,
    SphincsSha2128f,
}

impl SignatureScheme {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MlDsa65 => "ml-dsa-65",
            Self::SphincsSha2128f => "sphincs+-sha2-128f",
        }
    }
}

/// Key encapsulation mechanisms for node-session and transport bootstrap.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KemScheme {
    MlKem768,
}

impl KemScheme {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MlKem768 => "ml-kem-768",
        }
    }
}

/// Hash policy used for transaction signing domains and state commitments.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HashPolicy {
    Sha3_256,
    Blake3,
}

impl HashPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Sha3_256 => "sha3-256",
            Self::Blake3 => "blake3",
        }
    }
}

/// Quantum security profile validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantumProfileError {
    InvalidProfileVersion,
    EmptyAllowedSignatures,
    DefaultSignatureNotAllowed,
    FallbackSignatureNotAllowed,
    LegacySupportMustRemainDisabled,
}

impl fmt::Display for QuantumProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfileVersion => f.write_str("profile_version must be greater than zero"),
            Self::EmptyAllowedSignatures => {
                f.write_str("allowed_signatures must include at least one PQ signature scheme")
            }
            Self::DefaultSignatureNotAllowed => {
                f.write_str("default_signature must appear in allowed_signatures")
            }
            Self::FallbackSignatureNotAllowed => {
                f.write_str("fallback_signature must appear in allowed_signatures")
            }
            Self::LegacySupportMustRemainDisabled => {
                f.write_str("legacy_signature_support must remain disabled for strict profile")
            }
        }
    }
}

impl std::error::Error for QuantumProfileError {}

/// Canonical quantum-native kernel profile.
///
/// This profile is designed to be persisted in genesis and/or constitutional
/// config surfaces so that node behavior remains deterministic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuantumKernelProfile {
    pub profile_version: u16,
    pub default_signature: SignatureScheme,
    pub fallback_signature: Option<SignatureScheme>,
    pub allowed_signatures: Vec<SignatureScheme>,
    pub transport_kem: KemScheme,
    pub tx_hash_policy: HashPolicy,
    pub state_hash_policy: HashPolicy,
    pub legacy_signature_support: bool,
}

impl QuantumKernelProfile {
    /// Returns the strict quantum-native profile for AOXC pre-mainnet kernels.
    #[must_use]
    pub fn strict_default() -> Self {
        Self {
            profile_version: 2,
            default_signature: SignatureScheme::MlDsa65,
            fallback_signature: None,
            allowed_signatures: vec![SignatureScheme::MlDsa65],
            transport_kem: KemScheme::MlKem768,
            tx_hash_policy: HashPolicy::Sha3_256,
            state_hash_policy: HashPolicy::Blake3,
            legacy_signature_support: false,
        }
    }

    /// Validates profile consistency under fail-closed kernel policy rules.
    pub fn validate(&self) -> Result<(), QuantumProfileError> {
        if self.profile_version == 0 {
            return Err(QuantumProfileError::InvalidProfileVersion);
        }

        if self.allowed_signatures.is_empty() {
            return Err(QuantumProfileError::EmptyAllowedSignatures);
        }

        if !self.allowed_signatures.contains(&self.default_signature) {
            return Err(QuantumProfileError::DefaultSignatureNotAllowed);
        }

        if let Some(fallback_signature) = self.fallback_signature
            && !self.allowed_signatures.contains(&fallback_signature)
        {
            return Err(QuantumProfileError::FallbackSignatureNotAllowed);
        }

        if self.legacy_signature_support {
            return Err(QuantumProfileError::LegacySupportMustRemainDisabled);
        }

        Ok(())
    }
}
