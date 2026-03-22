use serde_json::{Map, Value, json};

use crate::{CanonicalizationError, ContractError, ContractManifest, Validate};

pub fn canonical_manifest_bytes(manifest: &ContractManifest) -> Result<Vec<u8>, ContractError> {
    manifest.validate()?;
    let value = serde_json::to_value(manifest)
        .map_err(|_| CanonicalizationError::CanonicalEncodingFailed)?;
    let normalized = canonicalize_value(value);
    serde_json::to_vec(&normalized)
        .map_err(|_| CanonicalizationError::CanonicalEncodingFailed.into())
}

pub fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut ordered = Map::new();
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in entries {
                ordered.insert(key, canonicalize_value(value));
            }
            Value::Object(ordered)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        Value::String(text) => Value::String(text.trim().to_string()),
        other => other,
    }
}

pub fn canonical_manifest_value(manifest: &ContractManifest) -> Result<Value, ContractError> {
    let bytes = canonical_manifest_bytes(manifest)?;
    serde_json::from_slice(&bytes).map_err(|_| {
        ContractError::Canonicalization(CanonicalizationError::CanonicalEncodingFailed)
    })
}

pub fn identity_projection(manifest: &ContractManifest) -> Result<Value, ContractError> {
    manifest.validate()?;
    Ok(canonicalize_value(json!({
        "schema_version": manifest.schema_version,
        "name": manifest.name,
        "package": manifest.package,
        "contract_version": manifest.contract_version,
        "vm_target": manifest.vm_target,
        "artifact_digest": manifest.artifact.artifact_digest,
        "digest": manifest.digest,
    })))
}
