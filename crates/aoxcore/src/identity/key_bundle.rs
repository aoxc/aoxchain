use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt;

use crate::identity::{
    certificate::Certificate,
    hd_path::{HdPath, HdPathError},
    key_engine::{DERIVED_ENTROPY_LEN, KeyEngine, KeyEngineError},
    keyfile::{KeyfileEnvelope, KeyfileError},
    passport::Passport,
};

/// Current canonical node key-bundle schema version.
pub const NODE_KEY_BUNDLE_VERSION: u8 = 1;

/// Supported cryptographic operating profiles for AOXC node bundles.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CryptoProfile {
    ClassicEd25519,
    HybridEd25519Dilithium3,
    PqDilithium3Preview,
}

impl CryptoProfile {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ClassicEd25519 => "classic-ed25519",
            Self::HybridEd25519Dilithium3 => "hybrid-ed25519-dilithium3",
            Self::PqDilithium3Preview => "pq-dilithium3-preview",
        }
    }
}

impl fmt::Display for CryptoProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Canonical operational roles that a node key can serve.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKeyRole {
    Identity,
    Consensus,
    Transport,
    Operator,
    Recovery,
    PqAttestation,
}

impl NodeKeyRole {
    #[must_use]
    pub const fn role_index(&self) -> u32 {
        match self {
            Self::Identity => 1,
            Self::Consensus => 2,
            Self::Transport => 3,
            Self::Operator => 4,
            Self::Recovery => 5,
            Self::PqAttestation => 6,
        }
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Consensus => "consensus",
            Self::Transport => "transport",
            Self::Operator => "operator",
            Self::Recovery => "recovery",
            Self::PqAttestation => "pq_attestation",
        }
    }
}

/// Typed public metadata for a single role-specific node key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeKeyRecord {
    pub role: NodeKeyRole,
    pub hd_path: String,
    pub algorithm: String,
    pub public_key: String,
    pub fingerprint: String,
}

/// Canonical node key bundle stored and consumed across AOXC operator surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeKeyBundleV1 {
    pub version: u8,
    pub node_name: String,
    pub profile: String,
    pub created_at: String,
    pub crypto_profile: CryptoProfile,
    pub custody_model: String,
    pub engine_fingerprint: String,
    pub bundle_fingerprint: String,
    pub encrypted_root_seed: KeyfileEnvelope,
    pub keys: Vec<NodeKeyRecord>,
}

/// Canonical error surface for node key-bundle operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum NodeKeyBundleError {
    EmptyNodeName,
    EmptyProfile,
    EmptyCreatedAt,
    EmptyCustodyModel,
    MissingKeys,
    MissingRole(NodeKeyRole),
    DuplicateRole(NodeKeyRole),
    EmptyPublicKey(NodeKeyRole),
    EmptyFingerprint(NodeKeyRole),
    EmptyHdPath(NodeKeyRole),
    InvalidPublicKeyHex(NodeKeyRole),
    InvalidPublicKeyLength(NodeKeyRole),
    UnauthorizedConsensusSigner,
    UnauthorizedTransportPeer,
    InvalidKeyfile(KeyfileError),
    SerializationFailed(String),
    InvalidHdPath(HdPathError),
    KeyDerivation(KeyEngineError),
}

impl fmt::Display for NodeKeyBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNodeName => {
                write!(f, "node key bundle validation failed: node_name is empty")
            }
            Self::EmptyProfile => write!(f, "node key bundle validation failed: profile is empty"),
            Self::EmptyCreatedAt => {
                write!(f, "node key bundle validation failed: created_at is empty")
            }
            Self::EmptyCustodyModel => {
                write!(
                    f,
                    "node key bundle validation failed: custody_model is empty"
                )
            }
            Self::MissingKeys => write!(f, "node key bundle validation failed: no keys present"),
            Self::MissingRole(role) => write!(
                f,
                "node key bundle validation failed: missing required role {}",
                role.as_str()
            ),
            Self::DuplicateRole(role) => write!(
                f,
                "node key bundle validation failed: duplicate role {}",
                role.as_str()
            ),
            Self::EmptyPublicKey(role) => write!(
                f,
                "node key bundle validation failed: public_key is empty for role {}",
                role.as_str()
            ),
            Self::EmptyFingerprint(role) => write!(
                f,
                "node key bundle validation failed: fingerprint is empty for role {}",
                role.as_str()
            ),
            Self::EmptyHdPath(role) => write!(
                f,
                "node key bundle validation failed: hd_path is empty for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyHex(role) => write!(
                f,
                "node key bundle validation failed: public_key hex is invalid for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyLength(role) => write!(
                f,
                "node key bundle validation failed: public_key length is invalid for role {}",
                role.as_str()
            ),
            Self::UnauthorizedConsensusSigner => {
                write!(
                    f,
                    "node key bundle authorization failed: consensus signer mismatch"
                )
            }
            Self::UnauthorizedTransportPeer => {
                write!(
                    f,
                    "node key bundle authorization failed: transport peer mismatch"
                )
            }
            Self::InvalidKeyfile(error) => {
                write!(f, "node key bundle validation failed: {}", error)
            }
            Self::SerializationFailed(error) => {
                write!(f, "node key bundle serialization failed: {}", error)
            }
            Self::InvalidHdPath(error) => {
                write!(f, "node key bundle path construction failed: {}", error)
            }
            Self::KeyDerivation(error) => write!(f, "node key bundle derivation failed: {}", error),
        }
    }
}

impl std::error::Error for NodeKeyBundleError {}

impl From<KeyEngineError> for NodeKeyBundleError {
    fn from(value: KeyEngineError) -> Self {
        Self::KeyDerivation(value)
    }
}

impl From<HdPathError> for NodeKeyBundleError {
    fn from(value: HdPathError) -> Self {
        Self::InvalidHdPath(value)
    }
}

impl NodeKeyBundleV1 {
    /// Builds a canonical key bundle from a key engine and encrypted root seed.
    pub fn generate(
        node_name: &str,
        profile: &str,
        created_at: String,
        crypto_profile: CryptoProfile,
        engine: &KeyEngine,
        encrypted_root_seed: KeyfileEnvelope,
    ) -> Result<Self, NodeKeyBundleError> {
        let keys = required_roles()
            .into_iter()
            .map(|role| build_record(engine, profile, &role))
            .collect::<Result<Vec<_>, _>>()?;

        let mut bundle = Self {
            version: NODE_KEY_BUNDLE_VERSION,
            node_name: node_name.to_string(),
            profile: profile.to_string(),
            created_at,
            crypto_profile,
            custody_model: "encrypted-root-seed-envelope".to_string(),
            engine_fingerprint: engine.fingerprint(),
            bundle_fingerprint: String::new(),
            encrypted_root_seed,
            keys,
        };

        bundle.bundle_fingerprint = bundle.compute_bundle_fingerprint()?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Validates the bundle shape and mandatory role presence.
    pub fn validate(&self) -> Result<(), NodeKeyBundleError> {
        if self.node_name.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyNodeName);
        }
        if self.profile.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyProfile);
        }
        if self.created_at.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyCreatedAt);
        }
        if self.custody_model.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyCustodyModel);
        }
        if self.keys.is_empty() {
            return Err(NodeKeyBundleError::MissingKeys);
        }

        let mut seen = Vec::new();
        for record in &self.keys {
            if record.hd_path.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyHdPath(record.role.clone()));
            }
            if record.public_key.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyPublicKey(record.role.clone()));
            }
            if record.fingerprint.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyFingerprint(record.role.clone()));
            }
            if seen.contains(&record.role) {
                return Err(NodeKeyBundleError::DuplicateRole(record.role.clone()));
            }
            seen.push(record.role.clone());
        }

        for role in required_roles() {
            if !seen.contains(&role) {
                return Err(NodeKeyBundleError::MissingRole(role));
            }
        }

        if let Err(error) =
            crate::identity::keyfile::decrypt_key_from_envelope(&self.encrypted_root_seed, " ")
        {
            match error {
                KeyfileError::EmptyPassword => {}
                other => return Err(NodeKeyBundleError::InvalidKeyfile(other)),
            }
        }

        Ok(())
    }

    /// Returns the canonical record for the requested role.
    #[must_use]
    pub fn key_record(&self, role: NodeKeyRole) -> Option<&NodeKeyRecord> {
        self.keys.iter().find(|record| record.role == role)
    }

    /// Returns the public key bytes for the requested role.
    pub fn public_key_bytes_for_role(
        &self,
        role: NodeKeyRole,
    ) -> Result<[u8; 32], NodeKeyBundleError> {
        let record = self
            .key_record(role.clone())
            .ok_or_else(|| NodeKeyBundleError::MissingRole(role.clone()))?;
        let decoded = hex::decode(&record.public_key)
            .map_err(|_| NodeKeyBundleError::InvalidPublicKeyHex(role.clone()))?;
        let bytes: [u8; 32] = decoded
            .as_slice()
            .try_into()
            .map_err(|_| NodeKeyBundleError::InvalidPublicKeyLength(role))?;
        Ok(bytes)
    }

    /// Returns the public key hex string for the requested role.
    pub fn public_key_hex_for_role(&self, role: NodeKeyRole) -> Result<&str, NodeKeyBundleError> {
        let record = self
            .key_record(role.clone())
            .ok_or_else(|| NodeKeyBundleError::MissingRole(role))?;
        Ok(record.public_key.as_str())
    }

    /// Validates whether the supplied signer matches the bundle consensus role.
    pub fn authorize_consensus_signer(&self, signer: [u8; 32]) -> Result<(), NodeKeyBundleError> {
        let expected = self.public_key_bytes_for_role(NodeKeyRole::Consensus)?;
        if signer == expected {
            Ok(())
        } else {
            Err(NodeKeyBundleError::UnauthorizedConsensusSigner)
        }
    }

    /// Validates whether the supplied producer identity matches the consensus role.
    pub fn authorize_block_producer(&self, producer: [u8; 32]) -> Result<(), NodeKeyBundleError> {
        self.authorize_consensus_signer(producer)
    }

    /// Validates whether the supplied peer identity matches the transport role.
    pub fn authorize_transport_peer(&self, peer: [u8; 32]) -> Result<(), NodeKeyBundleError> {
        let expected = self.public_key_bytes_for_role(NodeKeyRole::Transport)?;
        if peer == expected {
            Ok(())
        } else {
            Err(NodeKeyBundleError::UnauthorizedTransportPeer)
        }
    }

    /// Projects the canonical consensus role into an unsigned certificate.
    pub fn project_consensus_certificate(
        &self,
        chain: &str,
        actor_id: &str,
        zone: &str,
        issued_at: u64,
        expires_at: u64,
    ) -> Result<Certificate, NodeKeyBundleError> {
        let pubkey = self.public_key_hex_for_role(NodeKeyRole::Consensus)?;
        Ok(Certificate::new_unsigned(
            chain.to_string(),
            actor_id.to_string(),
            "validator".to_string(),
            zone.to_string(),
            pubkey.to_string(),
            issued_at,
            expires_at,
        ))
    }

    /// Projects the canonical consensus role into a runtime passport.
    pub fn project_validator_passport(
        &self,
        actor_id: &str,
        zone: &str,
        certificate_json: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Passport {
        Passport::new(
            actor_id.to_string(),
            "validator".to_string(),
            zone.to_string(),
            certificate_json,
            issued_at,
            expires_at,
        )
    }

    /// Serializes the bundle to pretty JSON.
    pub fn to_json(&self) -> Result<String, NodeKeyBundleError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))
    }

    /// Deserializes the bundle from JSON.
    pub fn from_json(data: &str) -> Result<Self, NodeKeyBundleError> {
        let bundle: Self = serde_json::from_str(data)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;
        bundle.validate()?;
        Ok(bundle)
    }

    fn compute_bundle_fingerprint(&self) -> Result<String, NodeKeyBundleError> {
        let canonical = serde_json::json!({
            "version": self.version,
            "node_name": self.node_name,
            "profile": self.profile,
            "created_at": self.created_at,
            "crypto_profile": self.crypto_profile.as_str(),
            "custody_model": self.custody_model,
            "engine_fingerprint": self.engine_fingerprint,
            "keys": self.keys,
        });

        let bytes = serde_json::to_vec(&canonical)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;
        let mut hasher = Sha3_256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        Ok(hex::encode_upper(&digest[..16]))
    }
}

fn required_roles() -> Vec<NodeKeyRole> {
    vec![
        NodeKeyRole::Identity,
        NodeKeyRole::Consensus,
        NodeKeyRole::Transport,
        NodeKeyRole::Operator,
        NodeKeyRole::Recovery,
        NodeKeyRole::PqAttestation,
    ]
}

fn build_record(
    engine: &KeyEngine,
    profile: &str,
    role: &NodeKeyRole,
) -> Result<NodeKeyRecord, NodeKeyBundleError> {
    let path = derive_role_path(profile, role)?;
    let material = engine.derive_key_material(&path)?;
    let public_key = derive_public_commitment(&material, role);
    let fingerprint = fingerprint_record(&public_key);

    Ok(NodeKeyRecord {
        role: role.clone(),
        hd_path: path.to_string_path(),
        algorithm: "AOXC-KeyEngine-Commitment-V1".to_string(),
        public_key,
        fingerprint,
    })
}

fn derive_role_path(profile: &str, role: &NodeKeyRole) -> Result<HdPath, NodeKeyBundleError> {
    let chain = match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => 1,
        "testnet" => 1001,
        "validator" => 2001,
        _ => 2626,
    };
    Ok(HdPath::new(chain, role.role_index(), 1, 0)?)
}

fn derive_public_commitment(material: &[u8; DERIVED_ENTROPY_LEN], role: &NodeKeyRole) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(b"AOXC-NODE-KEY-PUBLIC-COMMITMENT-V1");
    hasher.update([0x00]);
    hasher.update(role.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(material);
    hex::encode_upper(hasher.finalize())
}

fn fingerprint_record(public_key: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(public_key.as_bytes());
    let digest = hasher.finalize();
    hex::encode_upper(&digest[..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        key_engine::{KeyEngine, MASTER_SEED_LEN},
        keyfile::encrypt_key_to_envelope,
    };

    #[test]
    fn generated_bundle_contains_all_required_roles() {
        let engine = KeyEngine::from_seed([0x33; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");

        let bundle = NodeKeyBundleV1::generate(
            "validator-01",
            "testnet",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::HybridEd25519Dilithium3,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");

        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn bundle_roundtrip_preserves_bundle_fingerprint() {
        let engine = KeyEngine::from_seed([0x44; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");
        let bundle = NodeKeyBundleV1::generate(
            "validator-02",
            "mainnet",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::ClassicEd25519,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");
        let json = bundle.to_json().expect("json encoding must succeed");
        let decoded = NodeKeyBundleV1::from_json(&json).expect("json decoding must succeed");

        assert_eq!(bundle.bundle_fingerprint, decoded.bundle_fingerprint);
    }

    #[test]
    fn authorize_consensus_signer_accepts_matching_consensus_key() {
        let engine = KeyEngine::from_seed([0x55; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");
        let bundle = NodeKeyBundleV1::generate(
            "validator-03",
            "validator",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::HybridEd25519Dilithium3,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");

        let signer = bundle
            .public_key_bytes_for_role(NodeKeyRole::Consensus)
            .expect("consensus key must decode");

        assert!(bundle.authorize_consensus_signer(signer).is_ok());
    }

    #[test]
    fn project_consensus_certificate_uses_consensus_public_key() {
        let engine = KeyEngine::from_seed([0x77; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");
        let bundle = NodeKeyBundleV1::generate(
            "validator-04",
            "validator",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::HybridEd25519Dilithium3,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed");

        let cert = bundle
            .project_consensus_certificate("AOXC-TEST", "actor-1", "eu", 100, 200)
            .expect("certificate projection must succeed");

        assert_eq!(cert.role, "validator");
        assert_eq!(
            cert.pubkey,
            bundle
                .public_key_hex_for_role(NodeKeyRole::Consensus)
                .expect("consensus key must be present")
        );
    }
}
