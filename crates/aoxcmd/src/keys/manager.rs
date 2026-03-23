use crate::{
    error::AppError,
    keys::{
        loader::{load_operator_key, persist_operator_key},
        material::KeyMaterial,
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
    Ok(load_operator_key()?.fingerprint)
}

pub fn verify_operator_key(password: Option<&str>) -> Result<(), AppError> {
    let key = load_operator_key()?;
    if key.name.trim().is_empty()
        || key.public_key.trim().is_empty()
        || key.fingerprint.trim().is_empty()
        || key.hd_path.trim().is_empty()
    {
        return Err(crate::error::AppError::new(
            crate::error::ErrorCode::KeyMaterialInvalid,
            "Operator key material failed mandatory field validation",
        ));
    }
    let envelope = crate::keys::material::validate_key_envelope(&key.encrypted_private_key)?;
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
