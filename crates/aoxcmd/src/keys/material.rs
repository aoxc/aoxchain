// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::{AppError, ErrorCode};
use aoxcore::identity::{
    key_bundle::{CryptoProfile, NodeKeyBundleV1, NodeKeyRole},
    key_engine::KeyEngine,
    keyfile::{KeyfileEnvelope, encrypt_key_to_envelope},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

const OPERATIONAL_STATE_ACTIVE: &str = "active";
const OPERATIONAL_STATE_ROTATION_REQUIRED: &str = "rotation-required";
const OPERATIONAL_STATE_LOCKED: &str = "locked";
const DEFAULT_MAX_ACTIVE_AGE_DAYS: i64 = 90;

/// Canonical lifecycle state for persisted AOXC key material.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum KeyLifecycleState {
    Active,
    RotationRequired,
    Locked,
}

impl KeyLifecycleState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => OPERATIONAL_STATE_ACTIVE,
            Self::RotationRequired => OPERATIONAL_STATE_ROTATION_REQUIRED,
            Self::Locked => OPERATIONAL_STATE_LOCKED,
        }
    }
}

/// Canonical AOXC key-guard policy persisted with key material.
///
/// This policy enables deterministic operator-side automation:
/// - automatic lifecycle degradation after a maximum active age,
/// - emergency lockout toggles for compromised or suspended bundles,
/// - explicit profile policy for quantum-hardening posture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyGuardPolicy {
    pub max_active_age_days: i64,
    pub emergency_lock: bool,
    pub auto_rotate_enabled: bool,
}

impl Default for KeyGuardPolicy {
    fn default() -> Self {
        Self {
            max_active_age_days: DEFAULT_MAX_ACTIVE_AGE_DAYS,
            emergency_lock: false,
            auto_rotate_enabled: true,
        }
    }
}

/// Canonical persisted AOXC operator key material.
///
/// This structure intentionally stores the full AOXC node key bundle and
/// relies on the bundle's encrypted root-seed envelope for private-material
/// custody. No plaintext private key material is serialized here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub bundle: NodeKeyBundleV1,
    #[serde(default)]
    pub guard_policy: KeyGuardPolicy,
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
    pub key_lifecycle_state: String,
    pub key_age_days: i64,
    pub max_active_age_days: i64,
    pub auto_rotate_enabled: bool,
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

        let material = Self {
            bundle,
            guard_policy: KeyGuardPolicy::default(),
        };
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

        self.lifecycle_state()?;

        Ok(())
    }

    /// Builds an operational summary from the canonical node key bundle.
    pub fn summary(&self) -> Result<KeyMaterialSummary, AppError> {
        self.validate()?;
        let key_age_days = self.key_age_days()?;
        let key_lifecycle_state = self.lifecycle_state()?;

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
            key_lifecycle_state: key_lifecycle_state.as_str().to_string(),
            key_age_days,
            max_active_age_days: self.guard_policy.max_active_age_days,
            auto_rotate_enabled: self.guard_policy.auto_rotate_enabled,
            operational_state: key_lifecycle_state.as_str().to_string(),
            consensus_public_key: consensus_record.public_key.clone(),
            consensus_key_fingerprint: consensus_record.fingerprint.clone(),
            transport_public_key: transport_record.public_key.clone(),
            transport_key_fingerprint: transport_record.fingerprint.clone(),
        })
    }

    /// Returns computed key age in days from the bundle creation timestamp.
    pub fn key_age_days(&self) -> Result<i64, AppError> {
        let created_at = parse_created_at(&self.bundle.created_at)?;
        Ok((Utc::now() - created_at).num_days().max(0))
    }

    /// Computes the current lifecycle state from guard policy and key age.
    pub fn lifecycle_state(&self) -> Result<KeyLifecycleState, AppError> {
        if self.guard_policy.max_active_age_days <= 0 {
            return Ok(KeyLifecycleState::RotationRequired);
        }

        if self.guard_policy.emergency_lock {
            return Ok(KeyLifecycleState::Locked);
        }

        let created_at = parse_created_at(&self.bundle.created_at)?;
        let rotation_due = created_at + Duration::days(self.guard_policy.max_active_age_days);
        if Utc::now() >= rotation_due {
            return Ok(KeyLifecycleState::RotationRequired);
        }

        Ok(KeyLifecycleState::Active)
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
        "quantum" => Ok("mainnet"),
        "pq-preview" => Ok("mainnet"),
        other => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Unsupported AOXC key-material profile `{}`; expected mainnet, quantum, pq-preview, testnet, validation, devnet, or localnet",
                other
            ),
        )),
    }
}

fn parse_created_at(value: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Key bundle created_at must be a valid RFC3339 timestamp",
                error,
            )
        })
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
    use super::{KeyMaterial, validate_key_envelope};

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
        assert_eq!(summary.key_lifecycle_state, "active");
        assert!(summary.key_age_days >= 0);
        assert_eq!(summary.max_active_age_days, 90);
        assert!(summary.auto_rotate_enabled);
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
    fn quantum_profile_normalizes_to_mainnet_hybrid_surface() {
        let material = KeyMaterial::generate("validator-07", "quantum", "Quantum#2026!")
            .expect("quantum profile should succeed");

        assert_eq!(material.bundle.profile, "mainnet");
        assert_eq!(
            material.bundle.crypto_profile.as_str(),
            "hybrid-ed25519-dilithium3"
        );
    }

    #[test]
    fn summary_marks_rotation_required_when_key_age_exceeds_policy() {
        let mut material = KeyMaterial::generate("validator-08", "testnet", "Rotate#2026!")
            .expect("key generation should succeed");
        material.guard_policy.max_active_age_days = 0;

        let summary = material.summary().expect("summary should succeed");

        assert_eq!(summary.key_lifecycle_state, "rotation-required");
        assert_eq!(summary.operational_state, "rotation-required");
    }

    #[test]
    fn summary_marks_locked_when_emergency_lock_is_enabled() {
        let mut material = KeyMaterial::generate("validator-09", "mainnet", "Lock#2026!")
            .expect("key generation should succeed");
        material.guard_policy.emergency_lock = true;

        let summary = material.summary().expect("summary should succeed");
        assert_eq!(summary.key_lifecycle_state, "locked");
        assert_eq!(summary.operational_state, "locked");
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
