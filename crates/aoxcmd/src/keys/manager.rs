use crate::{
    error::AppError,
    keys::{
        loader::{load_operator_key, persist_operator_key},
        material::{ExportedIdentityArtifacts, KeyMaterial, KeyMaterialSummary},
    },
};

pub fn bootstrap_operator_key(
    name: &str,
    profile: &str,
    password: &str,
) -> Result<KeyMaterial, AppError> {
    let material = KeyMaterial::generate(name, profile, password)?;
    persist_operator_key(&material)?;
    Ok(material)
}

pub fn operator_fingerprint() -> Result<String, AppError> {
    Ok(load_operator_key()?.fingerprint().to_string())
}

pub fn consensus_public_key_hex() -> Result<String, AppError> {
    Ok(load_operator_key()?.consensus_public_key_hex()?.to_string())
}

pub fn inspect_operator_key() -> Result<KeyMaterialSummary, AppError> {
    load_operator_key()?.summary()
}

pub fn export_operator_identity(
    chain: &str,
    actor_id: &str,
    zone: &str,
    issued_at: u64,
    expires_at: u64,
) -> Result<ExportedIdentityArtifacts, AppError> {
    load_operator_key()?.export_validator_identity(chain, actor_id, zone, issued_at, expires_at)
}

pub fn verify_operator_key(password: Option<&str>) -> Result<(), AppError> {
    let key = load_operator_key()?;
    key.bundle.validate().map_err(|error| {
        crate::error::AppError::with_source(
            crate::error::ErrorCode::KeyMaterialInvalid,
            "Operator key bundle failed mandatory field validation",
            error,
        )
    })?;
    let serialized = serde_json::to_string_pretty(key.encrypted_root_seed()).map_err(|error| {
        crate::error::AppError::with_source(
            crate::error::ErrorCode::OutputEncodingFailed,
            "Failed to encode operator root-seed envelope",
            error,
        )
    })?;
    let envelope = crate::keys::material::validate_key_envelope(&serialized)?;
    if let Some(password) = password {
        aoxcore::identity::keyfile::decrypt_key_from_envelope(&envelope, password).map_err(
            |error| {
                crate::error::AppError::with_source(
                    crate::error::ErrorCode::KeyMaterialInvalid,
                    "Operator key decrypt verification failed",
                    error,
                )
            },
        )?;
    }
    Ok(())
}
