// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    data_home::{read_file, write_file},
    error::{AppError, ErrorCode},
    keys::{material::KeyMaterial, paths::operator_key_path},
};

/// Loads operator key material from the canonical AOXC operator-key path.
///
/// Validation contract:
/// - The file must exist and be readable.
/// - The payload must be valid JSON for the canonical `KeyMaterial` schema.
/// - The deserialized material must satisfy semantic validation before it is returned.
///
/// Error mapping policy:
/// - Missing file paths map to `KeyMaterialMissing`.
/// - Filesystem read failures map to `FilesystemIoFailed`.
/// - Decode and semantic validation failures map to `KeyMaterialInvalid`.
pub fn load_operator_key() -> Result<KeyMaterial, AppError> {
    let path = operator_key_path()?;
    let raw = read_file(&path).map_err(|error| {
        if error.has_io_error_kind(std::io::ErrorKind::NotFound) {
            AppError::new(
                ErrorCode::KeyMaterialMissing,
                format!("Operator key material is missing at {}", path.display()),
            )
        } else {
            AppError::with_source(
                ErrorCode::FilesystemIoFailed,
                format!(
                    "Failed to read operator key material from {}",
                    path.display()
                ),
                error,
            )
        }
    })?;

    let material: KeyMaterial = serde_json::from_str(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::KeyMaterialInvalid,
            format!(
                "Failed to parse operator key material at {}",
                path.display()
            ),
            error,
        )
    })?;

    validate_key_material(&material)?;
    Ok(material)
}

/// Persists operator key material to the canonical AOXC operator-key path.
///
/// Validation contract:
/// - Semantic validation is enforced before the payload is encoded.
/// - Only validated key material is serialized and written to disk.
///
/// Failure policy:
/// - Validation failures map to `KeyMaterialInvalid`.
/// - JSON encoding failures map to `OutputEncodingFailed`.
/// - Filesystem persistence failures are surfaced by the data-home layer.
pub fn persist_operator_key(material: &KeyMaterial) -> Result<(), AppError> {
    validate_key_material(material)?;

    let path = operator_key_path()?;
    let content = serde_json::to_string_pretty(material).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            format!(
                "Failed to encode operator key material for {}",
                path.display()
            ),
            error,
        )
    })?;

    write_file(&path, &content)
}

/// Enforces canonical semantic validation for AOXC operator key material.
fn validate_key_material(material: &KeyMaterial) -> Result<(), AppError> {
    material
        .validate()
        .map_err(|error| AppError::new(ErrorCode::KeyMaterialInvalid, error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{load_operator_key, persist_operator_key};
    use crate::{
        error::ErrorCode,
        keys::material::KeyMaterial,
        test_support::{AoxcHomeGuard, TestHome, aoxc_home_test_lock},
    };

    /// Executes a loader test inside a process-safe isolated AOXC home.
    fn with_test_home<T>(label: &str, test: impl FnOnce(&TestHome) -> T) -> T {
        let _lock = aoxc_home_test_lock();
        let home = TestHome::new(label);
        let _guard = AoxcHomeGuard::install(home.path());
        test(&home)
    }

    #[test]
    fn persist_and_reload_operator_key_round_trips_bundle() {
        with_test_home("keys-loader-roundtrip", |_home| {
            let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
                .expect("key material generation should succeed");
            let fingerprint = material.fingerprint().to_string();

            persist_operator_key(&material).expect("operator key should persist");
            let reloaded = load_operator_key().expect("persisted operator key should reload");

            assert_eq!(reloaded.fingerprint(), fingerprint);
            assert_eq!(reloaded.bundle.keys.len(), material.bundle.keys.len());
        });
    }

    #[test]
    fn load_operator_key_returns_missing_when_file_is_absent() {
        with_test_home("keys-loader-missing", |_home| {
            let error = load_operator_key().expect_err("missing operator key material must fail");

            assert_eq!(error.code(), ErrorCode::KeyMaterialMissing.as_str());
        });
    }

    #[test]
    fn load_operator_key_rejects_invalid_json() {
        with_test_home("keys-loader-invalid-json", |_home| {
            let path =
                crate::keys::paths::operator_key_path().expect("operator key path must resolve");
            crate::data_home::write_file(&path, "{ invalid json")
                .expect("fixture should be written");

            let error = load_operator_key().expect_err("invalid JSON must be rejected");

            assert_eq!(error.code(), ErrorCode::KeyMaterialInvalid.as_str());
        });
    }
}
