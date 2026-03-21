use crate::{
    data_home::{read_file, write_file},
    error::{AppError, ErrorCode},
    keys::{material::KeyMaterial, paths::operator_key_path},
};

pub fn load_operator_key() -> Result<KeyMaterial, AppError> {
    let path = operator_key_path()?;
    let raw = read_file(&path).map_err(|_| {
        AppError::new(
            ErrorCode::KeyMaterialMissing,
            format!("Operator key material is missing at {}", path.display()),
        )
    })?;
    serde_json::from_str(&raw)
        .map_err(|e| AppError::with_source(ErrorCode::KeyMaterialInvalid, "Failed to parse operator key material", e))
}

pub fn persist_operator_key(material: &KeyMaterial) -> Result<(), AppError> {
    let path = operator_key_path()?;
    let content = serde_json::to_string_pretty(material)
        .map_err(|e| AppError::with_source(ErrorCode::OutputEncodingFailed, "Failed to encode key material", e))?;
    write_file(&path, &content)
}
