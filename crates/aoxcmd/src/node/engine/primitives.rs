use super::*;

pub(super) fn proposer_key_from_material(key_material: &KeyMaterial) -> Result<[u8; 32], AppError> {
    let summary = key_material.summary()?;
    decode_hash32(
        &summary.consensus_public_key,
        "consensus_public_key",
        ErrorCode::KeyMaterialInvalid,
    )
}

pub(super) fn decode_hash32(
    value: &str,
    field: &str,
    code: ErrorCode,
) -> Result<[u8; 32], AppError> {
    let bytes = hex::decode(value)
        .map_err(|error| AppError::with_source(code, format!("Failed to decode {field}"), error))?;

    if bytes.len() != 32 {
        return Err(AppError::new(code, format!("{field} must be 32 bytes")));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub(super) fn derive_digest32(domain: &str, payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    hasher.finalize().into()
}

pub(super) fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(MINIMUM_RUNTIME_TIMESTAMP_UNIX)
        .max(MINIMUM_RUNTIME_TIMESTAMP_UNIX)
}
