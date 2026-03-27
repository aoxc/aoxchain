// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use aoxcore::identity::{
    key_bundle::{CryptoProfile, NodeKeyBundleV1, NodeKeyRole},
    key_engine::KeyEngine,
    keyfile::{encrypt_key_to_envelope, KeyfileEnvelope},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Canonical persisted AOXC operator key material.
///
/// This structure intentionally stores the full AOXC node key bundle and
/// relies on the bundle's encrypted root-seed envelope for private-material
/// custody. No plaintext private key material is serialized here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub bundle: NodeKeyBundleV1,
}

/// Canonical summary exported from persisted AOXC key material.
///
/// This summary is intended for:
/// - CLI inspection,
/// - validator metadata export,
/// - bootnode metadata export,
/// - operational fingerprint reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterialSummary {
    pub profile: String,
    pub bundle_fingerprint: String,
    pub operational_state: String,
    pub consensus_public_key: String,
    pub consensus_key_fingerprint: String,
    pub transport_public_key: String,
    pub transport_key_fingerprint: String,
}

impl KeyMaterial {
    /// Generates canonical AOXC key material for a node/operator surface.
    ///
    /// The resulting bundle:
    /// - uses a fresh deterministic engine root seed,
    /// - encrypts the root seed into a keyfile envelope,
    /// - derives canonical role-scoped public keys,
    /// - stores only public bundle metadata plus encrypted seed custody.
    pub fn generate(name: &str, profile: &str, password: &str) -> Result<Self, AppError> {
        let normalized_profile = normalize_profile(profile)?;
        let created_at = Utc::now().to_rfc3339();
        let engine = KeyEngine::new(None);

        let encrypted_root_seed =
            encrypt_key_to_envelope(engine.master_seed(), password).map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to encrypt AOXC root seed into canonical keyfile envelope",
                    error,
                )
            })?;

        let bundle = NodeKeyBundleV1::generate(
            name,
            normalized_profile,
            created_at,
            infer_crypto_profile(normalized_profile),
            &engine,
            encrypted_root_seed,
        )
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to build canonical AOXC node key bundle",
                error,
            )
        })?;

        Ok(Self { bundle })
    }

    /// Returns the canonical bundle fingerprint.
    pub fn fingerprint(&self) -> &str {
        &self.bundle.bundle_fingerprint
    }

    /// Returns the encrypted root-seed envelope.
    pub fn encrypted_root_seed(&self) -> &KeyfileEnvelope {
        &self.bundle.encrypted_root_seed
    }

    /// Builds an operational summary from the canonical node key bundle.
    pub fn summary(&self) -> Result<KeyMaterialSummary, AppError> {
        self.bundle.validate().map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to validate AOXC node key bundle while building key summary",
                error,
            )
        })?;

        let consensus_record = self
            .bundle
            .keys
            .iter()
            .find(|record| matches!(record.role, NodeKeyRole::Consensus))
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::KeyMaterialInvalid,
                    "Consensus key record is missing from AOXC node key bundle",
                )
            })?;

        let transport_record = self
            .bundle
            .keys
            .iter()
            .find(|record| matches!(record.role, NodeKeyRole::Transport))
            .ok_or_else(|| {
                AppError::new(
                    ErrorCode::KeyMaterialInvalid,
                    "Transport key record is missing from AOXC node key bundle",
                )
            })?;

        Ok(KeyMaterialSummary {
            profile: self.bundle.profile.clone(),
            bundle_fingerprint: self.bundle.bundle_fingerprint.clone(),
            operational_state: "active".to_string(),
            consensus_public_key: consensus_record.public_key.clone(),
            consensus_key_fingerprint: consensus_record.fingerprint.clone(),
            transport_public_key: transport_record.public_key.clone(),
            transport_key_fingerprint: transport_record.fingerprint.clone(),
        })
    }
}

/// Infers the canonical AOXC cryptographic operating profile from the
/// normalized environment profile.
///
/// Current policy:
/// - mainnet => hybrid surface reservation,
/// - testnet / validation / devnet / localnet => classic Ed25519 operational mode.
///
/// This policy can be tightened later without changing the consumer contract.
fn infer_crypto_profile(profile: &str) -> CryptoProfile {
    match profile {
        "mainnet" => CryptoProfile::HybridEd25519Dilithium3,
        "testnet" | "validation" | "devnet" | "localnet" => CryptoProfile::ClassicEd25519,
        _ => CryptoProfile::HybridEd25519Dilithium3,
    }
}

/// Normalizes accepted key-generation profile names.
fn normalize_profile(profile: &str) -> Result<&'static str, AppError> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => Ok("mainnet"),
        "testnet" => Ok("testnet"),
        "validation" => Ok("validation"),
        "validator" => Ok("validation"),
        "devnet" => Ok("devnet"),
        "localnet" => Ok("localnet"),
        other => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Unsupported AOXC key-material profile `{}`; expected mainnet, testnet, validation, devnet, or localnet",
                other
            ),
        )),
    }
}

/// Validates a serialized AOXC keyfile envelope.
pub fn validate_key_envelope(serialized: &str) -> Result<KeyfileEnvelope, AppError> {
    serde_json::from_str(serialized).map_err(|error| {
        AppError::with_source(
            ErrorCode::KeyMaterialInvalid,
            "Stored operator key envelope is malformed",
            error,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{validate_key_envelope, KeyMaterial};

    #[test]
    fn generated_material_uses_canonical_node_key_bundle() {
        let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
            .expect("key generation should succeed");

        assert_eq!(material.bundle.version, 2);
        assert_eq!(material.bundle.profile, "testnet");
        assert_eq!(material.bundle.keys.len(), 6);
        assert_eq!(
            material.bundle.custody_model,
            "encrypted-root-seed-envelope"
        );
        assert!(material.bundle.keys[0].hd_path.starts_with("m/44/2626/"));

        let serialized = serde_json::to_string_pretty(material.encrypted_root_seed())
            .expect("envelope serialization should succeed");
        assert!(validate_key_envelope(&serialized).is_ok());
    }

    #[test]
    fn generated_material_summary_contains_consensus_and_transport_data() {
        let material = KeyMaterial::generate("validator-02", "validation", "Test#2026!")
            .expect("key generation should succeed");

        let summary = material
            .summary()
            .expect("summary generation should succeed");

        assert_eq!(summary.profile, "validation");
        assert_eq!(summary.operational_state, "active");
        assert!(!summary.bundle_fingerprint.is_empty());
        assert!(!summary.consensus_public_key.is_empty());
        assert!(!summary.consensus_key_fingerprint.is_empty());
        assert!(!summary.transport_public_key.is_empty());
        assert!(!summary.transport_key_fingerprint.is_empty());
        assert_ne!(summary.consensus_public_key, summary.transport_public_key);
    }

    #[test]
    fn validator_alias_normalizes_to_validation_profile() {
        let material = KeyMaterial::generate("validator-03", "validator", "Alias#2026!")
            .expect("legacy validator alias should succeed");

        assert_eq!(material.bundle.profile, "validation");
    }

    #[test]
    fn unsupported_profile_is_rejected() {
        let result = KeyMaterial::generate("validator-04", "staging", "Fail#2026!");
        assert!(result.is_err());
    }
}
