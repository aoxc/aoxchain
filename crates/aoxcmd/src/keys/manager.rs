// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    error::{AppError, ErrorCode},
    keys::{
        loader::{load_operator_key, persist_operator_key},
        material::{KeyMaterial, KeyMaterialSummary},
    },
};

/// Canonical AOXC CLI key-management profile normalization.
///
/// This function preserves backward compatibility for the legacy `validator`
/// profile name while enforcing the modern canonical profile vocabulary used
/// across AOXC configuration, identity, and bootstrap surfaces.
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
                "Unsupported AOXC key-management profile `{}`; expected mainnet, testnet, validation, devnet, or localnet",
                other
            ),
        )),
    }
}

/// Generates and persists canonical AOXC operator key material.
///
/// The generated material is profile-aware and becomes the active operator key
/// bundle on disk after successful persistence.
pub fn bootstrap_operator_key(
    name: &str,
    profile: &str,
    password: &str,
) -> Result<KeyMaterial, AppError> {
    let profile = normalize_profile(profile)?;
    let material = KeyMaterial::generate(name, profile, password)?;
    persist_operator_key(&material)?;
    Ok(material)
}

/// Rotates the currently persisted AOXC operator key bundle.
///
/// Rotation preserves the invariant that a prior operator bundle must already
/// exist before a replacement is written.
pub fn rotate_operator_key(
    name: &str,
    profile: &str,
    password: &str,
) -> Result<KeyMaterialSummary, AppError> {
    let profile = normalize_profile(profile)?;
    let _previous = load_operator_key()?;
    let rotated = KeyMaterial::generate(name, profile, password)?;
    persist_operator_key(&rotated)?;
    rotated.summary()
}

/// Returns the active operator bundle fingerprint.
pub fn operator_fingerprint() -> Result<String, AppError> {
    Ok(load_operator_key()?.fingerprint().to_string())
}

/// Returns the canonical summary of the active operator bundle.
pub fn inspect_operator_key() -> Result<KeyMaterialSummary, AppError> {
    load_operator_key()?.summary()
}

/// Verifies the active operator key bundle.
///
/// Verification always performs structural bundle validation. If a password is
/// supplied, the encrypted root-seed envelope is also decrypted as a live
/// custody verification step.
pub fn verify_operator_key(password: Option<&str>) -> Result<(), AppError> {
    let key = load_operator_key()?;

    key.bundle.validate().map_err(|error| {
        AppError::with_source(
            ErrorCode::KeyMaterialInvalid,
            "Operator key bundle failed mandatory field validation",
            error,
        )
    })?;

    let serialized = serde_json::to_string_pretty(key.encrypted_root_seed()).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to encode operator root-seed envelope",
            error,
        )
    })?;

    let envelope = crate::keys::material::validate_key_envelope(&serialized)?;

    if let Some(password) = password {
        aoxcore::identity::keyfile::decrypt_key_from_envelope(&envelope, password).map_err(
            |error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Operator key decrypt verification failed",
                    error,
                )
            },
        )?;
    }

    Ok(())
}

/// Returns the active consensus public key as uppercase hexadecimal.
///
/// This function reads the current persisted operator bundle summary and exposes
/// the consensus role public key for environment bootstrap and validator export
/// workflows.
pub fn consensus_public_key_hex() -> Result<String, AppError> {
    Ok(inspect_operator_key()?.consensus_public_key)
}

/// Returns the active transport public key as uppercase hexadecimal.
///
/// This function is intended for:
/// - bootnode export,
/// - peer identity publication,
/// - AOXC network transport metadata generation.
pub fn transport_public_key_hex() -> Result<String, AppError> {
    Ok(inspect_operator_key()?.transport_public_key)
}

/// Returns a fully validated active key summary.
///
/// This helper is useful for callers that want a single validated summary
/// surface before exporting data into environment bundle files.
pub fn validated_operator_key_summary() -> Result<KeyMaterialSummary, AppError> {
    verify_operator_key(None)?;
    inspect_operator_key()
}

#[cfg(test)]
mod tests {
    use super::{
        bootstrap_operator_key, consensus_public_key_hex, inspect_operator_key,
        operator_fingerprint, rotate_operator_key, transport_public_key_hex,
        validated_operator_key_summary, verify_operator_key,
    };
    use crate::test_support::TestHome;

    #[test]
    fn bootstrap_operator_key_persists_inspectable_material() {
        let _home = TestHome::new("bootstrap-operator-key");

        let material = bootstrap_operator_key("validator-01", "testnet", "Test#2026!")
            .expect("operator key bootstrap should succeed");
        let summary = inspect_operator_key().expect("bootstrapped key should be inspectable");

        assert_eq!(
            operator_fingerprint().expect("fingerprint should load"),
            material.fingerprint()
        );
        assert_eq!(
            consensus_public_key_hex().expect("consensus public key should load"),
            summary.consensus_public_key
        );
        assert_eq!(
            transport_public_key_hex().expect("transport public key should load"),
            summary.transport_public_key
        );

        verify_operator_key(Some("Test#2026!"))
            .expect("bootstrapped operator key should verify with the bootstrap password");
    }

    #[test]
    fn rotate_operator_key_replaces_fingerprint_and_keeps_key_valid() {
        let _home = TestHome::new("rotate-operator-key");

        let original = bootstrap_operator_key("validator-01", "testnet", "Test#2026!")
            .expect("initial operator key bootstrap should succeed");
        let rotated = rotate_operator_key("validator-02", "mainnet", "Rotate#2026!")
            .expect("operator key rotation should succeed");

        assert_ne!(rotated.bundle_fingerprint, original.fingerprint());
        assert_eq!(rotated.operational_state, "active");

        verify_operator_key(Some("Rotate#2026!"))
            .expect("rotated operator key should verify with the new password");
    }

    #[test]
    fn legacy_validator_profile_alias_is_accepted() {
        let _home = TestHome::new("bootstrap-legacy-validator-alias");

        let material = bootstrap_operator_key("validator-03", "validator", "Alias#2026!")
            .expect("legacy validator alias should normalize to validation");

        let summary = material.summary().expect("summary must be available");
        assert_eq!(summary.profile, "validation");
    }

    #[test]
    fn unsupported_profile_is_rejected() {
        let _home = TestHome::new("bootstrap-unsupported-profile");

        let result = bootstrap_operator_key("validator-04", "staging", "Fail#2026!");
        assert!(result.is_err());
    }

    #[test]
    fn transport_and_consensus_public_keys_are_distinct() {
        let _home = TestHome::new("distinct-consensus-and-transport");

        bootstrap_operator_key("validator-05", "validation", "Distinct#2026!")
            .expect("bootstrap should succeed");

        let summary = inspect_operator_key().expect("summary should load");
        assert_ne!(summary.consensus_public_key, summary.transport_public_key);
    }

    #[test]
    fn validated_operator_key_summary_runs_structural_validation() {
        let _home = TestHome::new("validated-operator-summary");

        bootstrap_operator_key("validator-06", "devnet", "Summary#2026!")
            .expect("bootstrap should succeed");

        let summary =
            validated_operator_key_summary().expect("validated summary retrieval should succeed");

        assert_eq!(summary.profile, "devnet");
        assert!(!summary.bundle_fingerprint.is_empty());
    }

    #[test]
    fn profile_normalization_maps_validator_to_validation() {
        let normalized =
            super::normalize_profile("validator").expect("legacy alias must normalize");
        assert_eq!(normalized, "validation");
    }
}
