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

const OPERATIONAL_STATE_ACTIVE: &str = "active";

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
    /// Generation contract:
    /// - `name` must not be blank.
    /// - `profile` must resolve to a canonical AOXC profile.
    /// - `password` must not be blank.
    /// - The encrypted root-seed envelope is persisted inside the generated
    ///   node key bundle.
    ///
    /// Security rationale:
    /// - Only encrypted seed custody is retained.
    /// - The generated bundle is validated before it is returned.
    pub fn generate(name: &str, profile: &str, password: &str) -> Result<Self, AppError> {
        let normalized_name = normalize_required_text(name, "name")?;
        let normalized_profile = normalize_profile(profile)?;
        let normalized_password = normalize_required_text(password, "password")?;
        let created_at = Utc::now().to_rfc3339();

        let engine = KeyEngine::new(None);

        let encrypted_root_seed =
            encrypt_key_to_envelope(engine.master_seed(), &normalized_password).map_err(
                |error| {
                    AppError::with_source(
                        ErrorCode::KeyMaterialInvalid,
                        "Failed to encrypt AOXC root seed into canonical keyfile envelope",
                        error,
                    )
                },
            )?;

        let bundle = NodeKeyBundleV1::generate(
            &normalized_name,
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

        let material = Self { bundle };
        material.validate()?;
        Ok(material)
    }

    /// Returns the canonical bundle fingerprint.
    pub fn fingerprint(&self) -> &str {
        &self.bundle.bundle_fingerprint
    }

    /// Returns the encrypted root-seed envelope.
    pub fn encrypted_root_seed(&self) -> &KeyfileEnvelope {
        &self.bundle.encrypted_root_seed
    }

    /// Performs canonical semantic validation over persisted AOXC key material.
    ///
    /// Validation policy:
    /// - The bundle must satisfy its own structural validation contract.
    /// - The profile must remain inside the canonical AOXC profile vocabulary.
    /// - Mandatory role records required by the operator plane must exist.
    pub fn validate(&self) -> Result<(), AppError> {
        self.bundle.validate().map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to validate AOXC node key bundle",
                error,
            )
        })?;

        normalize_profile(&self.bundle.profile)?;

        let has_consensus = self
            .bundle
            .keys
            .iter()
            .any(|record| matches!(record.role, NodeKeyRole::Consensus));
        if !has_consensus {
            return Err(AppError::new(
                ErrorCode::KeyMaterialInvalid,
                "Consensus key record is missing from AOXC node key bundle",
            ));
        }

        let has_transport = self
            .bundle
            .keys
            .iter()
            .any(|record| matches!(record.role, NodeKeyRole::Transport));
        if !has_transport {
            return Err(AppError::new(
                ErrorCode::KeyMaterialInvalid,
                "Transport key record is missing from AOXC node key bundle",
            ));
        }

        Ok(())
    }

    /// Builds an operational summary from the canonical node key bundle.
    pub fn summary(&self) -> Result<KeyMaterialSummary, AppError> {
        self.validate()?;

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
            operational_state: OPERATIONAL_STATE_ACTIVE.to_string(),
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
fn infer_crypto_profile(profile: &str) -> CryptoProfile {
    match profile {
        "mainnet" => CryptoProfile::HybridEd25519Dilithium3,
        "testnet" | "validation" | "devnet" | "localnet" => CryptoProfile::ClassicEd25519,
        _ => unreachable!("profile must be normalized before crypto profile inference"),
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

/// Enforces non-blank normalized operator-facing text input.
fn normalize_required_text(value: &str, field: &str) -> Result<String, AppError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("AOXC key-material {} must not be blank", field),
        ));
    }
    Ok(normalized)
}

/// Validates a serialized AOXC keyfile envelope.
///
/// Validation policy:
/// - The serialized payload must not be blank.
/// - The payload must decode into the canonical keyfile envelope schema.
pub fn validate_key_envelope(serialized: &str) -> Result<KeyfileEnvelope, AppError> {
    if serialized.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::KeyMaterialInvalid,
            "Stored operator key envelope must not be blank",
        ));
    }

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

    #[test]
    fn blank_name_is_rejected() {
        let result = KeyMaterial::generate("   ", "testnet", "Test#2026!");
        assert!(result.is_err());
    }

    #[test]
    fn blank_password_is_rejected() {
        let result = KeyMaterial::generate("validator-05", "testnet", "   ");
        assert!(result.is_err());
    }

    #[test]
    fn validate_key_envelope_rejects_blank_payload() {
        let result = validate_key_envelope("   ");
        assert!(result.is_err());
    }

    #[test]
    fn validate_method_accepts_generated_material() {
        let material = KeyMaterial::generate("validator-06", "devnet", "Test#2026!")
            .expect("key generation should succeed");

        assert!(material.validate().is_ok());
    }
}
