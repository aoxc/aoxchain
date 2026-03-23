use crate::error::{AppError, ErrorCode};
use aoxcore::identity::{
    key_bundle::{CryptoProfile, NodeKeyBundleV1},
    key_engine::KeyEngine,
    keyfile::{encrypt_key_to_envelope, KeyfileEnvelope},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMaterial {
    pub bundle: NodeKeyBundleV1,
}

impl KeyMaterial {
    pub fn generate(name: &str, profile: &str, password: &str) -> Result<Self, AppError> {
        let created_at = Utc::now().to_rfc3339();
        let engine = KeyEngine::new(None);
        let encrypted_root_seed =
            encrypt_key_to_envelope(engine.master_seed(), password).map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to protect operator key material",
                    error,
                )
            })?;
        let bundle = NodeKeyBundleV1::generate(
            name,
            profile,
            created_at,
            infer_crypto_profile(profile),
            &engine,
            encrypted_root_seed,
        )
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::KeyMaterialInvalid,
                "Failed to build canonical node key bundle",
                error,
            )
        })?;

        Ok(Self { bundle })
    }

    pub fn fingerprint(&self) -> &str {
        &self.bundle.bundle_fingerprint
    }

    pub fn encrypted_root_seed(&self) -> &KeyfileEnvelope {
        &self.bundle.encrypted_root_seed
    }

    pub fn consensus_public_key_hex(&self) -> Result<&str, AppError> {
        self.bundle
            .public_key_hex_for_role(aoxcore::identity::key_bundle::NodeKeyRole::Consensus)
            .map_err(|error| {
                AppError::with_source(
                    ErrorCode::KeyMaterialInvalid,
                    "Failed to read canonical consensus public key from key bundle",
                    error,
                )
            })
    }
}

fn infer_crypto_profile(profile: &str) -> CryptoProfile {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => CryptoProfile::HybridEd25519Dilithium3,
        "testnet" | "validator" => CryptoProfile::ClassicEd25519,
        _ => CryptoProfile::HybridEd25519Dilithium3,
    }
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
    fn generated_material_uses_canonical_node_key_bundle() {
        let material = KeyMaterial::generate("validator-01", "testnet", "Test#2026!")
            .expect("key generation should succeed");

        assert_eq!(material.bundle.version, 1);
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
}
