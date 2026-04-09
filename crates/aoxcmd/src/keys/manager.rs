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
///
/// Accepted canonical values:
/// - `mainnet`
/// - `quantum`
/// - `quntum`
/// - `qumtum`
/// - `pq-preview`
/// - `testnet`
/// - `validation`
/// - `devnet`
/// - `localnet`
///
/// Backward-compatible alias:
/// - `validator` -> `validation`
fn normalize_profile(profile: &str) -> Result<&'static str, AppError> {
    let normalized = profile.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "mainnet" => Ok("mainnet"),
        "testnet" => Ok("testnet"),
        "validation" => Ok("validation"),
        "validator" => Ok("validation"),
        "devnet" => Ok("devnet"),
        "localnet" => Ok("localnet"),
        "quantum" => Ok("mainnet"),
        "quntum" => Ok("mainnet"),
        "qumtum" => Ok("mainnet"),
        "pq-preview" => Ok("mainnet"),
        other => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Unsupported AOXC key-management profile `{}`; expected mainnet, quantum, quntum, qumtum, pq-preview, testnet, validation, devnet, or localnet",
                other
            ),
        )),
    }
}

/// Generates and persists canonical AOXC operator key material.
///
/// Validation policy:
/// - `name` must not be blank.
/// - `password` must not be blank.
/// - `profile` must resolve to a canonical AOXC profile.
///
/// Persistence contract:
/// - The generated bundle becomes the active operator key bundle only after
///   successful persistence to the canonical operator-key path.
pub fn bootstrap_operator_key(
    name: &str,
    profile: &str,
    password: &str,
) -> Result<KeyMaterial, AppError> {
    let normalized_name = normalize_required_text(name, "name")?;
    let normalized_profile = normalize_profile(profile)?;
    let normalized_password = normalize_required_text(password, "password")?;

    let material =
        KeyMaterial::generate(&normalized_name, normalized_profile, &normalized_password)?;
    persist_operator_key(&material)?;
    Ok(material)
}

/// Rotates the currently persisted AOXC operator key bundle.
///
/// Rotation policy:
/// - A prior operator bundle must already exist and pass validation.
/// - The replacement bundle must be generated and persisted successfully
///   before the new summary is returned.
pub fn rotate_operator_key(
    name: &str,
    profile: &str,
    password: &str,
) -> Result<KeyMaterialSummary, AppError> {
    let normalized_name = normalize_required_text(name, "name")?;
    let normalized_profile = normalize_profile(profile)?;
    let normalized_password = normalize_required_text(password, "password")?;

    let previous = load_operator_key()?;
    validate_loaded_key_material(&previous)?;

    let rotated =
        KeyMaterial::generate(&normalized_name, normalized_profile, &normalized_password)?;
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
/// Verification always performs:
/// 1. full key-material validation,
/// 2. bundle-level structural validation,
/// 3. root-seed envelope validation.
///
/// If a password is supplied, the encrypted root-seed envelope is also
/// decrypted as a live custody verification step.
pub fn verify_operator_key(password: Option<&str>) -> Result<(), AppError> {
    let key = load_operator_key()?;
    validate_loaded_key_material(&key)?;

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
        let normalized_password = normalize_required_text(password, "password")?;
        aoxcore::identity::keyfile::decrypt_key_from_envelope(&envelope, &normalized_password)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Operator key decrypt verification failed",
                    error,
                )
            })?;
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

/// Enforces non-blank normalized text input for operator-facing arguments.
fn normalize_required_text(value: &str, field: &str) -> Result<String, AppError> {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Operator key {} must not be blank", field),
        ));
    }
    Ok(normalized)
}

/// Performs canonical semantic validation over loaded key material.
fn validate_loaded_key_material(material: &KeyMaterial) -> Result<(), AppError> {
    material
        .validate()
        .map_err(|error| AppError::new(ErrorCode::KeyMaterialInvalid, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        bootstrap_operator_key, consensus_public_key_hex, inspect_operator_key, normalize_profile,
        operator_fingerprint, rotate_operator_key, transport_public_key_hex,
        validated_operator_key_summary, verify_operator_key,
    };
    use crate::{
        error::ErrorCode,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn bootstrap_operator_key_persists_inspectable_material() {
        with_test_home("bootstrap-operator-key", |_home| {
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
        });
    }

    #[test]
    fn rotate_operator_key_replaces_fingerprint_and_keeps_key_valid() {
        with_test_home("rotate-operator-key", |_home| {
            let original = bootstrap_operator_key("validator-01", "testnet", "Test#2026!")
                .expect("initial operator key bootstrap should succeed");
            let rotated = rotate_operator_key("validator-02", "mainnet", "Rotate#2026!")
                .expect("operator key rotation should succeed");

            assert_ne!(rotated.bundle_fingerprint, original.fingerprint());
            assert_eq!(rotated.operational_state, "active");

            verify_operator_key(Some("Rotate#2026!"))
                .expect("rotated operator key should verify with the new password");
        });
    }

    #[test]
    fn legacy_validator_profile_alias_is_accepted() {
        with_test_home("bootstrap-legacy-validator-alias", |_home| {
            let material = bootstrap_operator_key("validator-03", "validator", "Alias#2026!")
                .expect("legacy validator alias should normalize to validation");

            let summary = material.summary().expect("summary must be available");
            assert_eq!(summary.profile, "validation");
        });
    }

    #[test]
    fn unsupported_profile_is_rejected() {
        with_test_home("bootstrap-unsupported-profile", |_home| {
            let error = bootstrap_operator_key("validator-04", "staging", "Fail#2026!")
                .expect_err("unsupported profile must fail");

            assert_eq!(error.code(), ErrorCode::UsageInvalidArguments.as_str());
        });
    }

    #[test]
    fn transport_and_consensus_public_keys_are_distinct() {
        with_test_home("distinct-consensus-and-transport", |_home| {
            bootstrap_operator_key("validator-05", "validation", "Distinct#2026!")
                .expect("bootstrap should succeed");

            let summary = inspect_operator_key().expect("summary should load");
            assert_ne!(summary.consensus_public_key, summary.transport_public_key);
        });
    }

    #[test]
    fn validated_operator_key_summary_runs_structural_validation() {
        with_test_home("validated-operator-summary", |_home| {
            bootstrap_operator_key("validator-06", "devnet", "Summary#2026!")
                .expect("bootstrap should succeed");

            let summary = validated_operator_key_summary()
                .expect("validated summary retrieval should succeed");

            assert_eq!(summary.profile, "devnet");
            assert!(!summary.bundle_fingerprint.is_empty());
        });
    }

    #[test]
    fn profile_normalization_maps_validator_to_validation() {
        let normalized = normalize_profile("validator").expect("legacy alias must normalize");
        assert_eq!(normalized, "validation");
    }

    #[test]
    fn profile_normalization_accepts_quantum_profiles() {
        assert_eq!(
            normalize_profile("quantum").expect("quantum profile should normalize"),
            "mainnet"
        );
        assert_eq!(
            normalize_profile("quntum").expect("quntum profile should normalize"),
            "mainnet"
        );
        assert_eq!(
            normalize_profile("pq-preview").expect("pq-preview profile should normalize"),
            "mainnet"
        );
        assert_eq!(
            normalize_profile("qumtum").expect("qumtum profile should normalize"),
            "mainnet"
        );
    }

    #[test]
    fn bootstrap_operator_key_rejects_blank_name() {
        with_test_home("bootstrap-blank-name", |_home| {
            let error = bootstrap_operator_key("   ", "testnet", "Test#2026!")
                .expect_err("blank name must fail");

            assert_eq!(error.code(), ErrorCode::UsageInvalidArguments.as_str());
        });
    }

    #[test]
    fn verify_operator_key_rejects_blank_password_when_password_check_is_requested() {
        with_test_home("verify-blank-password", |_home| {
            bootstrap_operator_key("validator-07", "testnet", "Verify#2026!")
                .expect("bootstrap should succeed");

            let error = verify_operator_key(Some("   "))
                .expect_err("blank password verification must fail");

            assert_eq!(error.code(), ErrorCode::UsageInvalidArguments.as_str());
        });
    }
}
