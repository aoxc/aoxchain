use crate::error::{AppError, ErrorCode};
use aoxcore::identity::{
    hd_path::HdPath,
    key_engine::KeyEngine,
    keyfile::{encrypt_key_to_envelope, KeyfileEnvelope},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub version: u8,
    pub name: String,
    pub profile: String,
    pub created_at: String,
    pub hd_path: String,
    pub key_algorithm: String,
    pub custody_model: String,
    pub fingerprint: String,
    pub public_key: String,
    pub encrypted_private_key: String,
}

impl KeyMaterial {
    pub fn generate(name: &str, profile: &str, password: &str) -> Result<Self, AppError> {
        let created_at = Utc::now().to_rfc3339();
        let engine = KeyEngine::new(None);
        let hd_path = derive_hd_path(profile).map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to construct canonical HD path for operator key",
                error,
            )
        })?;
        let derived_key = engine.derive_key_material(&hd_path).map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to derive operator key material",
                error,
            )
        })?;
        let encrypted_private_key = serialize_encrypted_master_seed(engine.master_seed(), password)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to protect operator key material",
                    error,
                )
            })?;

        let mut public_hasher = Sha3_256::new();
        public_hasher.update(name.as_bytes());
        public_hasher.update(profile.as_bytes());
        public_hasher.update(created_at.as_bytes());
        public_hasher.update(derived_key);
        let public_key = hex::encode_upper(public_hasher.finalize());

        let mut fp_hasher = Sha3_256::new();
        fp_hasher.update(public_key.as_bytes());
        let fingerprint_full = hex::encode(fp_hasher.finalize());

        Ok(Self {
            version: 2,
            name: name.to_string(),
            profile: profile.to_string(),
            created_at,
            hd_path: hd_path.to_string_path(),
            key_algorithm: "AOXC-KeyEngine-Seeded".to_string(),
            custody_model: "encrypted-master-seed-envelope".to_string(),
            fingerprint: fingerprint_full[..16].to_string(),
            public_key,
            encrypted_private_key,
        })
    }
}

fn derive_hd_path(profile: &str) -> Result<HdPath, aoxcore::identity::hd_path::HdPathError> {
    let chain = match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => 1,
        "testnet" => 1001,
        "validator" => 2001,
        _ => 2626,
    };
    HdPath::new(chain, 1, 1, 0)
}

fn serialize_encrypted_master_seed(
    master_seed: &[u8],
    password: &str,
) -> Result<String, aoxcore::identity::keyfile::KeyfileError> {
    let envelope = encrypt_key_to_envelope(master_seed, password)?;
    serde_json::to_string_pretty(&envelope).map_err(|error| {
        aoxcore::identity::keyfile::KeyfileError::SerializationFailed(error.to_string())
    })
}

pub fn validate_key_envelope(serialized: &str) -> Result<KeyfileEnvelope, AppError> {
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
    fn generated_material_uses_encrypted_seed_envelope() {
        let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
            .expect("key generation should succeed");

        assert_eq!(material.version, 2);
        assert_eq!(material.key_algorithm, "AOXC-KeyEngine-Seeded");
        assert_eq!(material.custody_model, "encrypted-master-seed-envelope");
        assert!(material.hd_path.starts_with("m/44/2626/"));
        assert!(validate_key_envelope(&material.encrypted_private_key).is_ok());
    }
}
