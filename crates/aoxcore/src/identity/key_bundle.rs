// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::BTreeSet;
use std::fmt;

use crate::identity::{
    hd_path::{HdPath, HdPathError},
    key_engine::{DERIVED_ENTROPY_LEN, KeyEngine, KeyEngineError},
    keyfile::{KeyfileEnvelope, KeyfileError, validate_envelope},
};

/// Current canonical node key-bundle schema version.
///
/// Compatibility note:
/// the struct name is retained for compatibility with the current codebase,
/// but the canonical serialized schema version remains explicitly versioned here.
pub const NODE_KEY_BUNDLE_VERSION: u8 = 2;

/// Canonical AOXC public key encoding used inside serialized key bundles.
pub const AOXC_PUBLIC_KEY_ENCODING: &str = "hex";

/// Canonical AOXC public key length for Ed25519 verifying keys.
pub const AOXC_ED25519_PUBLIC_KEY_LEN: usize = 32;

/// Canonical custody model string for node key bundles.
pub const AOXC_NODE_KEY_CUSTODY_MODEL: &str = "encrypted-root-seed-envelope";

/// Canonical domain used for deterministic Ed25519 role-key derivation inside bundles.
const AOXC_NODE_BUNDLE_ED25519_ROLE_SEED_DOMAIN: &[u8] = b"AOXC/NODE_BUNDLE/ED25519/ROLE_SEED/V1";

/// Canonical domain used for public-key fingerprints inside bundles.
const AOXC_NODE_BUNDLE_PUBLIC_KEY_FINGERPRINT_DOMAIN: &[u8] =
    b"AOXC/NODE_BUNDLE/PUBLIC_KEY_FINGERPRINT/V1";

/// Canonical domain used for bundle fingerprint derivation.
const AOXC_NODE_BUNDLE_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/NODE_BUNDLE/FINGERPRINT/V1";

/// Expected engine fingerprint length in uppercase hexadecimal characters.
const ENGINE_FINGERPRINT_HEX_LEN: usize = 32;

/// Expected bundle fingerprint length in uppercase hexadecimal characters.
const BUNDLE_FINGERPRINT_HEX_LEN: usize = 32;

/// Supported cryptographic operating profiles for AOXC node bundles.
///
/// At the current baseline:
/// - `ClassicEd25519` produces operational Ed25519 keys for all required roles.
/// - `HybridEd25519Dilithium3` preserves the same Ed25519 operational surface
///   while reserving the profile for future dual-surface augmentation.
/// - `PqDilithium3Preview` is retained as a compatibility placeholder, but
///   operational node roles still expose Ed25519 public material until the
///   downstream PQ operational interfaces are finalized across the stack.
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

    /// Returns the currently exposed operational public-key algorithm.
    ///
    /// This intentionally reflects the bundle’s serialized operational key
    /// surface rather than the full future cryptographic ambition of the
    /// profile name.
    #[must_use]
    pub const fn operational_public_key_algorithm(&self) -> &'static str {
        "ed25519"
    }
}

impl fmt::Display for CryptoProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Canonical operational roles that a node key can serve.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    /// Returns the canonical AOXC HD role index.
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

    #[must_use]
    pub const fn all() -> [NodeKeyRole; 6] {
        [
            Self::Identity,
            Self::Consensus,
            Self::Transport,
            Self::Operator,
            Self::Recovery,
            Self::PqAttestation,
        ]
    }
}

/// Typed public metadata for a single role-specific node key.
///
/// This record intentionally contains only public metadata. Private key
/// material is never serialized in plaintext here. Operational recovery of
/// role keys is expected to occur deterministically from the encrypted root
/// seed and the canonical HD path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeKeyRecord {
    pub role: NodeKeyRole,
    pub hd_path: String,
    pub algorithm: String,
    pub public_key_encoding: String,
    pub public_key: String,
    pub fingerprint: String,
}

/// Canonical node key bundle stored and consumed across AOXC operator surfaces.
///
/// This bundle is designed to provide:
/// - encrypted root-seed custody,
/// - deterministic role-key derivation,
/// - public key distribution metadata,
/// - stable bundle fingerprinting for audit workflows.
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
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NodeKeyBundleError {
    InvalidVersion,
    EmptyNodeName,
    EmptyProfile,
    EmptyCreatedAt,
    EmptyCustodyModel,
    EmptyEngineFingerprint,
    EmptyBundleFingerprint,
    InvalidEngineFingerprint,
    InvalidBundleFingerprint,
    BundleFingerprintMismatch,
    MissingKeys,
    MissingRole(NodeKeyRole),
    DuplicateRole(NodeKeyRole),
    EmptyPublicKey(NodeKeyRole),
    EmptyFingerprint(NodeKeyRole),
    EmptyHdPath(NodeKeyRole),
    InvalidAlgorithm(NodeKeyRole),
    InvalidPublicKeyEncoding(NodeKeyRole),
    InvalidPublicKeyHex(NodeKeyRole),
    InvalidPublicKeyMaterial(NodeKeyRole),
    InvalidPublicKeyLength {
        role: NodeKeyRole,
        expected: usize,
        actual: usize,
    },
    FingerprintMismatch(NodeKeyRole),
    HdPathMismatch {
        role: NodeKeyRole,
        expected: String,
        actual: String,
    },
    InvalidKeyfile(KeyfileError),
    SerializationFailed(String),
    InvalidHdPath(HdPathError),
    KeyDerivation(KeyEngineError),
    UnsupportedProfile(String),
}

impl NodeKeyBundleError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidVersion => "NODE_KEY_BUNDLE_INVALID_VERSION",
            Self::EmptyNodeName => "NODE_KEY_BUNDLE_EMPTY_NODE_NAME",
            Self::EmptyProfile => "NODE_KEY_BUNDLE_EMPTY_PROFILE",
            Self::EmptyCreatedAt => "NODE_KEY_BUNDLE_EMPTY_CREATED_AT",
            Self::EmptyCustodyModel => "NODE_KEY_BUNDLE_EMPTY_CUSTODY_MODEL",
            Self::EmptyEngineFingerprint => "NODE_KEY_BUNDLE_EMPTY_ENGINE_FINGERPRINT",
            Self::EmptyBundleFingerprint => "NODE_KEY_BUNDLE_EMPTY_BUNDLE_FINGERPRINT",
            Self::InvalidEngineFingerprint => "NODE_KEY_BUNDLE_INVALID_ENGINE_FINGERPRINT",
            Self::InvalidBundleFingerprint => "NODE_KEY_BUNDLE_INVALID_BUNDLE_FINGERPRINT",
            Self::BundleFingerprintMismatch => "NODE_KEY_BUNDLE_FINGERPRINT_MISMATCH",
            Self::MissingKeys => "NODE_KEY_BUNDLE_MISSING_KEYS",
            Self::MissingRole(_) => "NODE_KEY_BUNDLE_MISSING_ROLE",
            Self::DuplicateRole(_) => "NODE_KEY_BUNDLE_DUPLICATE_ROLE",
            Self::EmptyPublicKey(_) => "NODE_KEY_BUNDLE_EMPTY_PUBLIC_KEY",
            Self::EmptyFingerprint(_) => "NODE_KEY_BUNDLE_EMPTY_FINGERPRINT",
            Self::EmptyHdPath(_) => "NODE_KEY_BUNDLE_EMPTY_HD_PATH",
            Self::InvalidAlgorithm(_) => "NODE_KEY_BUNDLE_INVALID_ALGORITHM",
            Self::InvalidPublicKeyEncoding(_) => "NODE_KEY_BUNDLE_INVALID_PUBLIC_KEY_ENCODING",
            Self::InvalidPublicKeyHex(_) => "NODE_KEY_BUNDLE_INVALID_PUBLIC_KEY_HEX",
            Self::InvalidPublicKeyMaterial(_) => "NODE_KEY_BUNDLE_INVALID_PUBLIC_KEY_MATERIAL",
            Self::InvalidPublicKeyLength { .. } => "NODE_KEY_BUNDLE_INVALID_PUBLIC_KEY_LENGTH",
            Self::FingerprintMismatch(_) => "NODE_KEY_BUNDLE_FINGERPRINT_MISMATCH_FOR_ROLE",
            Self::HdPathMismatch { .. } => "NODE_KEY_BUNDLE_HD_PATH_MISMATCH",
            Self::InvalidKeyfile(_) => "NODE_KEY_BUNDLE_INVALID_KEYFILE",
            Self::SerializationFailed(_) => "NODE_KEY_BUNDLE_SERIALIZATION_FAILED",
            Self::InvalidHdPath(_) => "NODE_KEY_BUNDLE_INVALID_HD_PATH",
            Self::KeyDerivation(_) => "NODE_KEY_BUNDLE_KEY_DERIVATION_FAILED",
            Self::UnsupportedProfile(_) => "NODE_KEY_BUNDLE_UNSUPPORTED_PROFILE",
        }
    }
}

impl fmt::Display for NodeKeyBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion => {
                write!(f, "node key bundle validation failed: version is invalid")
            }
            Self::EmptyNodeName => {
                write!(f, "node key bundle validation failed: node_name is empty")
            }
            Self::EmptyProfile => {
                write!(f, "node key bundle validation failed: profile is empty")
            }
            Self::EmptyCreatedAt => {
                write!(f, "node key bundle validation failed: created_at is empty")
            }
            Self::EmptyCustodyModel => {
                write!(
                    f,
                    "node key bundle validation failed: custody_model is empty"
                )
            }
            Self::EmptyEngineFingerprint => {
                write!(
                    f,
                    "node key bundle validation failed: engine_fingerprint is empty"
                )
            }
            Self::EmptyBundleFingerprint => {
                write!(
                    f,
                    "node key bundle validation failed: bundle_fingerprint is empty"
                )
            }
            Self::InvalidEngineFingerprint => {
                write!(
                    f,
                    "node key bundle validation failed: engine_fingerprint is invalid"
                )
            }
            Self::InvalidBundleFingerprint => {
                write!(
                    f,
                    "node key bundle validation failed: bundle_fingerprint is invalid"
                )
            }
            Self::BundleFingerprintMismatch => {
                write!(
                    f,
                    "node key bundle validation failed: bundle_fingerprint mismatch"
                )
            }
            Self::MissingKeys => {
                write!(f, "node key bundle validation failed: no keys present")
            }
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
            Self::InvalidAlgorithm(role) => write!(
                f,
                "node key bundle validation failed: algorithm is invalid for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyEncoding(role) => write!(
                f,
                "node key bundle validation failed: unsupported public_key_encoding for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyHex(role) => write!(
                f,
                "node key bundle validation failed: public key hex is invalid for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyMaterial(role) => write!(
                f,
                "node key bundle validation failed: public key material is invalid for role {}",
                role.as_str()
            ),
            Self::InvalidPublicKeyLength {
                role,
                expected,
                actual,
            } => write!(
                f,
                "node key bundle validation failed: public key length mismatch for role {}; expected {} bytes, got {} bytes",
                role.as_str(),
                expected,
                actual
            ),
            Self::FingerprintMismatch(role) => write!(
                f,
                "node key bundle validation failed: fingerprint mismatch for role {}",
                role.as_str()
            ),
            Self::HdPathMismatch {
                role,
                expected,
                actual,
            } => write!(
                f,
                "node key bundle validation failed: hd_path mismatch for role {}; expected `{}`, got `{}`",
                role.as_str(),
                expected,
                actual
            ),
            Self::InvalidKeyfile(error) => {
                write!(f, "node key bundle validation failed: {}", error)
            }
            Self::SerializationFailed(error) => {
                write!(f, "node key bundle serialization failed: {}", error)
            }
            Self::InvalidHdPath(error) => {
                write!(f, "node key bundle path construction failed: {}", error)
            }
            Self::KeyDerivation(error) => {
                write!(f, "node key bundle derivation failed: {}", error)
            }
            Self::UnsupportedProfile(profile) => write!(
                f,
                "node key bundle derivation failed: unsupported profile `{}`",
                profile
            ),
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

impl From<KeyfileError> for NodeKeyBundleError {
    fn from(value: KeyfileError) -> Self {
        Self::InvalidKeyfile(value)
    }
}

impl NodeKeyBundleV1 {
    /// Builds a canonical key bundle from a key engine and encrypted root seed.
    ///
    /// The generated bundle:
    /// - derives deterministic role-specific Ed25519 keypairs,
    /// - stores only public metadata for each role,
    /// - preserves encrypted custody of the root seed through the supplied envelope.
    pub fn generate(
        node_name: &str,
        profile: &str,
        created_at: String,
        crypto_profile: CryptoProfile,
        engine: &KeyEngine,
        encrypted_root_seed: KeyfileEnvelope,
    ) -> Result<Self, NodeKeyBundleError> {
        let normalized_profile = normalize_profile(profile)?;

        let mut keys = NodeKeyRole::all()
            .into_iter()
            .map(|role| build_record(engine, normalized_profile, &crypto_profile, role))
            .collect::<Result<Vec<_>, _>>()?;

        keys.sort_by_key(|record| record.role);

        let mut bundle = Self {
            version: NODE_KEY_BUNDLE_VERSION,
            node_name: node_name.to_string(),
            profile: normalized_profile.to_string(),
            created_at,
            crypto_profile,
            custody_model: AOXC_NODE_KEY_CUSTODY_MODEL.to_string(),
            engine_fingerprint: engine.fingerprint(),
            bundle_fingerprint: String::new(),
            encrypted_root_seed,
            keys,
        };

        bundle.bundle_fingerprint = bundle.compute_bundle_fingerprint()?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Validates the bundle shape, role completeness, key metadata, and fingerprint integrity.
    pub fn validate(&self) -> Result<(), NodeKeyBundleError> {
        if self.version != NODE_KEY_BUNDLE_VERSION {
            return Err(NodeKeyBundleError::InvalidVersion);
        }

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

        if self.custody_model != AOXC_NODE_KEY_CUSTODY_MODEL {
            return Err(NodeKeyBundleError::EmptyCustodyModel);
        }

        if self.engine_fingerprint.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyEngineFingerprint);
        }

        if !is_uppercase_hex_with_len(&self.engine_fingerprint, ENGINE_FINGERPRINT_HEX_LEN) {
            return Err(NodeKeyBundleError::InvalidEngineFingerprint);
        }

        if self.bundle_fingerprint.trim().is_empty() {
            return Err(NodeKeyBundleError::EmptyBundleFingerprint);
        }

        if !is_uppercase_hex_with_len(&self.bundle_fingerprint, BUNDLE_FINGERPRINT_HEX_LEN) {
            return Err(NodeKeyBundleError::InvalidBundleFingerprint);
        }

        if self.keys.is_empty() {
            return Err(NodeKeyBundleError::MissingKeys);
        }

        let normalized_profile = normalize_profile(&self.profile)?;
        if normalized_profile != self.profile {
            return Err(NodeKeyBundleError::UnsupportedProfile(self.profile.clone()));
        }

        validate_envelope(&self.encrypted_root_seed)?;

        let mut seen_roles = BTreeSet::new();

        for record in &self.keys {
            if !seen_roles.insert(record.role) {
                return Err(NodeKeyBundleError::DuplicateRole(record.role));
            }

            if record.hd_path.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyHdPath(record.role));
            }

            if record.algorithm != self.crypto_profile.operational_public_key_algorithm() {
                return Err(NodeKeyBundleError::InvalidAlgorithm(record.role));
            }

            if record.public_key_encoding != AOXC_PUBLIC_KEY_ENCODING {
                return Err(NodeKeyBundleError::InvalidPublicKeyEncoding(record.role));
            }

            if record.public_key.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyPublicKey(record.role));
            }

            if record.fingerprint.trim().is_empty() {
                return Err(NodeKeyBundleError::EmptyFingerprint(record.role));
            }

            if !record
                .public_key
                .chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase())
            {
                return Err(NodeKeyBundleError::InvalidPublicKeyHex(record.role));
            }

            let decoded = hex::decode(&record.public_key)
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyHex(record.role))?;

            if decoded.len() != AOXC_ED25519_PUBLIC_KEY_LEN {
                return Err(NodeKeyBundleError::InvalidPublicKeyLength {
                    role: record.role,
                    expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                    actual: decoded.len(),
                });
            }

            let public_key_bytes: [u8; AOXC_ED25519_PUBLIC_KEY_LEN] = decoded
                .as_slice()
                .try_into()
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyLength {
                    role: record.role,
                    expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                    actual: decoded.len(),
                })?;

            let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
                .map_err(|_| NodeKeyBundleError::InvalidPublicKeyMaterial(record.role))?;

            let expected_fingerprint = fingerprint_record(&verifying_key);
            if record.fingerprint != expected_fingerprint {
                return Err(NodeKeyBundleError::FingerprintMismatch(record.role));
            }

            let parsed_path: HdPath = record.hd_path.parse()?;
            let expected_path = derive_role_path(&self.profile, record.role)?;

            if parsed_path != expected_path {
                return Err(NodeKeyBundleError::HdPathMismatch {
                    role: record.role,
                    expected: expected_path.to_string(),
                    actual: parsed_path.to_string(),
                });
            }
        }

        for role in NodeKeyRole::all() {
            if !seen_roles.contains(&role) {
                return Err(NodeKeyBundleError::MissingRole(role));
            }
        }

        let expected_bundle_fingerprint = self.compute_bundle_fingerprint()?;
        if self.bundle_fingerprint != expected_bundle_fingerprint {
            return Err(NodeKeyBundleError::BundleFingerprintMismatch);
        }

        Ok(())
    }

    /// Serializes the bundle to pretty JSON.
    pub fn to_json(&self) -> Result<String, NodeKeyBundleError> {
        serde_json::to_string_pretty(self)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))
    }

    /// Deserializes the bundle from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, NodeKeyBundleError> {
        let bundle: Self = serde_json::from_str(data)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;
        bundle.validate()?;
        Ok(bundle)
    }

    /// Returns the raw Ed25519 public-key bytes for the requested role.
    pub fn public_key_bytes_for_role(
        &self,
        role: NodeKeyRole,
    ) -> Result<[u8; AOXC_ED25519_PUBLIC_KEY_LEN], NodeKeyBundleError> {
        let record = self
            .keys
            .iter()
            .find(|record| record.role == role)
            .ok_or(NodeKeyBundleError::MissingRole(role))?;

        let bytes = hex::decode(&record.public_key)
            .map_err(|_| NodeKeyBundleError::InvalidPublicKeyHex(role))?;

        if bytes.len() != AOXC_ED25519_PUBLIC_KEY_LEN {
            return Err(NodeKeyBundleError::InvalidPublicKeyLength {
                role,
                expected: AOXC_ED25519_PUBLIC_KEY_LEN,
                actual: bytes.len(),
            });
        }

        let mut out = [0u8; AOXC_ED25519_PUBLIC_KEY_LEN];
        out.copy_from_slice(&bytes);
        Ok(out)
    }

    /// Computes the canonical bundle fingerprint.
    ///
    /// Fingerprint policy:
    /// - binds public and custody metadata,
    /// - excludes the stored `bundle_fingerprint` field itself,
    /// - keeps deterministic ordering as serialized in `keys`.
    fn compute_bundle_fingerprint(&self) -> Result<String, NodeKeyBundleError> {
        let canonical = serde_json::json!({
            "version": self.version,
            "node_name": self.node_name,
            "profile": self.profile,
            "created_at": self.created_at,
            "crypto_profile": self.crypto_profile.as_str(),
            "custody_model": self.custody_model,
            "engine_fingerprint": self.engine_fingerprint,
            "encrypted_root_seed": self.encrypted_root_seed,
            "keys": self.keys,
        });

        let bytes = serde_json::to_vec(&canonical)
            .map_err(|error| NodeKeyBundleError::SerializationFailed(error.to_string()))?;

        let mut hasher = Sha3_256::new();
        hasher.update(AOXC_NODE_BUNDLE_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(bytes);

        let digest = hasher.finalize();
        Ok(hex::encode_upper(&digest[..16]))
    }
}

/// Normalizes accepted profile aliases into canonical AOXC profile names.
///
/// Current canonical names:
/// - `mainnet`
/// - `testnet`
/// - `validation`
/// - `devnet`
/// - `localnet`
///
/// Backward-compatible aliases:
/// - `validator` => `validation`
fn normalize_profile(profile: &str) -> Result<&'static str, NodeKeyBundleError> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "mainnet" => Ok("mainnet"),
        "testnet" => Ok("testnet"),
        "validation" => Ok("validation"),
        "validator" => Ok("validation"),
        "devnet" => Ok("devnet"),
        "localnet" => Ok("localnet"),
        other => Err(NodeKeyBundleError::UnsupportedProfile(other.to_string())),
    }
}

/// Derives the canonical AOXC HD path for the requested operational profile and role.
///
/// Important compatibility note:
/// these chain identifiers are intentionally kept within the canonical unhardened
/// 31-bit HD component range so they remain compatible with strict `HdPath` validation.
fn derive_role_path(profile: &str, role: NodeKeyRole) -> Result<HdPath, NodeKeyBundleError> {
    let normalized = normalize_profile(profile)?;

    let chain = match normalized {
        "mainnet" => 26_260_001,
        "testnet" => 26_260_101,
        "validation" => 26_260_301,
        "devnet" => 26_260_201,
        "localnet" => 26_269_001,
        _ => {
            return Err(NodeKeyBundleError::UnsupportedProfile(
                normalized.to_string(),
            ));
        }
    };

    Ok(HdPath::new(chain, role.role_index(), 1, 0)?)
}

/// Builds a canonical public node-key record for the requested role.
fn build_record(
    engine: &KeyEngine,
    profile: &str,
    crypto_profile: &CryptoProfile,
    role: NodeKeyRole,
) -> Result<NodeKeyRecord, NodeKeyBundleError> {
    let path = derive_role_path(profile, role)?;
    let material = engine.derive_key_material(&path)?;
    let signing_key = derive_ed25519_signing_key(&material, role);
    let verifying_key: VerifyingKey = signing_key.verifying_key();

    Ok(NodeKeyRecord {
        role,
        hd_path: path.to_string(),
        algorithm: crypto_profile
            .operational_public_key_algorithm()
            .to_string(),
        public_key_encoding: AOXC_PUBLIC_KEY_ENCODING.to_string(),
        public_key: hex::encode_upper(verifying_key.to_bytes()),
        fingerprint: fingerprint_record(&verifying_key),
    })
}

/// Derives a deterministic Ed25519 signing key from canonical AOXC role material.
///
/// The derivation process intentionally avoids reusing the 64-byte engine output
/// directly as an Ed25519 expanded secret. Instead, it compresses the role-scoped
/// material through a dedicated domain-separated hash and uses the first 32 bytes
/// as the Ed25519 seed material.
fn derive_ed25519_signing_key(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role: NodeKeyRole,
) -> SigningKey {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_NODE_BUNDLE_ED25519_ROLE_SEED_DOMAIN);
    hasher.update([0x00]);
    hasher.update(role.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(material);

    let digest = hasher.finalize();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&digest[..32]);

    SigningKey::from_bytes(&seed)
}

/// Derives a stable short fingerprint from an Ed25519 verifying key.
fn fingerprint_record(public_key: &VerifyingKey) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_NODE_BUNDLE_PUBLIC_KEY_FINGERPRINT_DOMAIN);
    hasher.update([0x00]);
    hasher.update(public_key.to_bytes());

    let digest = hasher.finalize();
    hex::encode_upper(&digest[..8])
}

/// Returns whether the provided string is uppercase hexadecimal of an exact length.
fn is_uppercase_hex_with_len(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::{
        key_engine::{KeyEngine, MASTER_SEED_LEN},
        keyfile::encrypt_key_to_envelope,
    };

    fn make_bundle(
        seed_byte: u8,
        node_name: &str,
        profile: &str,
        crypto_profile: CryptoProfile,
    ) -> NodeKeyBundleV1 {
        let engine = KeyEngine::from_seed([seed_byte; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");

        NodeKeyBundleV1::generate(
            node_name,
            profile,
            "2026-01-01T00:00:00Z".to_string(),
            crypto_profile,
            &engine,
            envelope,
        )
        .expect("bundle generation must succeed")
    }

    #[test]
    fn generated_bundle_contains_all_required_roles() {
        let bundle = make_bundle(
            0x33,
            "validator-01",
            "testnet",
            CryptoProfile::HybridEd25519Dilithium3,
        );

        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn bundle_roundtrip_preserves_bundle_fingerprint() {
        let bundle = make_bundle(
            0x44,
            "validator-02",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let json = bundle.to_json().expect("json encoding must succeed");
        let decoded = NodeKeyBundleV1::from_json(&json).expect("json decoding must succeed");

        assert_eq!(bundle.bundle_fingerprint, decoded.bundle_fingerprint);
    }

    #[test]
    fn generated_public_keys_have_ed25519_length() {
        let bundle = make_bundle(
            0x55,
            "validator-03",
            "validation",
            CryptoProfile::ClassicEd25519,
        );

        for record in &bundle.keys {
            let decoded = hex::decode(&record.public_key).expect("public key must be valid hex");
            assert_eq!(decoded.len(), AOXC_ED25519_PUBLIC_KEY_LEN);
            assert_eq!(record.public_key_encoding, AOXC_PUBLIC_KEY_ENCODING);
            assert_eq!(record.algorithm, "ed25519");
        }
    }

    #[test]
    fn different_roles_produce_distinct_public_keys() {
        let bundle = make_bundle(
            0x66,
            "validator-04",
            "devnet",
            CryptoProfile::ClassicEd25519,
        );

        let identity = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        let consensus = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist");

        assert_ne!(identity.public_key, consensus.public_key);
    }

    #[test]
    fn validator_profile_is_accepted_as_validation_alias() {
        let bundle = make_bundle(
            0x77,
            "validator-05",
            "validator",
            CryptoProfile::ClassicEd25519,
        );

        assert_eq!(bundle.profile, "validation");
        assert_eq!(bundle.keys.len(), 6);
        assert!(bundle.validate().is_ok());
    }

    #[test]
    fn unsupported_profile_is_rejected() {
        let engine = KeyEngine::from_seed([0x88; MASTER_SEED_LEN]);
        let envelope =
            encrypt_key_to_envelope(engine.master_seed(), "Test#2026!").expect("must encrypt");

        let result = NodeKeyBundleV1::generate(
            "validator-06",
            "unknown-profile",
            "2026-01-01T00:00:00Z".to_string(),
            CryptoProfile::ClassicEd25519,
            &engine,
            envelope,
        );

        assert!(matches!(
            result,
            Err(NodeKeyBundleError::UnsupportedProfile(_))
        ));
    }

    #[test]
    fn same_seed_same_profile_same_role_material_is_stable() {
        let bundle_a = make_bundle(
            0x99,
            "validator-07",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );
        let bundle_b = make_bundle(
            0x99,
            "validator-07",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        assert_eq!(bundle_a.keys, bundle_b.keys);
        assert_eq!(bundle_a.bundle_fingerprint, bundle_b.bundle_fingerprint);
    }

    #[test]
    fn different_profiles_produce_distinct_consensus_public_keys() {
        let mainnet_bundle = make_bundle(
            0xAA,
            "validator-08",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );
        let testnet_bundle = make_bundle(
            0xAA,
            "validator-08",
            "testnet",
            CryptoProfile::ClassicEd25519,
        );

        let mainnet_consensus = mainnet_bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("mainnet consensus role must exist");

        let testnet_consensus = testnet_bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("testnet consensus role must exist");

        assert_ne!(mainnet_consensus.public_key, testnet_consensus.public_key);
    }

    #[test]
    fn public_key_bytes_are_available_for_every_required_role() {
        let bundle = make_bundle(
            0xBB,
            "validator-09",
            "localnet",
            CryptoProfile::ClassicEd25519,
        );

        for role in NodeKeyRole::all() {
            let bytes = bundle
                .public_key_bytes_for_role(role)
                .expect("public key bytes must exist for each role");
            assert_eq!(bytes.len(), AOXC_ED25519_PUBLIC_KEY_LEN);
        }
    }

    #[test]
    fn validate_rejects_duplicate_role_records() {
        let mut bundle = make_bundle(
            0xCC,
            "validator-10",
            "validation",
            CryptoProfile::ClassicEd25519,
        );

        let duplicate = bundle
            .keys
            .iter()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist")
            .clone();

        bundle.keys.push(duplicate);

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::DuplicateRole(NodeKeyRole::Consensus))
        ));
    }

    #[test]
    fn validate_rejects_tampered_hd_path() {
        let mut bundle = make_bundle(
            0xDD,
            "validator-11",
            "testnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Consensus)
            .expect("consensus role must exist");

        record.hd_path = "m/44/2626/1/2/1/0".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::HdPathMismatch { .. })
        ));
    }

    #[test]
    fn validate_rejects_invalid_public_key_encoding() {
        let mut bundle = make_bundle(
            0xEE,
            "validator-12",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        record.public_key_encoding = "base64".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::InvalidPublicKeyEncoding(
                NodeKeyRole::Identity
            ))
        ));
    }

    #[test]
    fn validate_rejects_tampered_record_fingerprint() {
        let mut bundle = make_bundle(
            0x11,
            "validator-13",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        let record = bundle
            .keys
            .iter_mut()
            .find(|record| record.role == NodeKeyRole::Identity)
            .expect("identity role must exist");

        record.fingerprint = "AAAAAAAAAAAAAAAA".to_string();

        let result = bundle.validate();
        assert!(matches!(
            result,
            Err(NodeKeyBundleError::FingerprintMismatch(
                NodeKeyRole::Identity
            ))
        ));
    }

    #[test]
    fn validate_rejects_bundle_fingerprint_mismatch() {
        let mut bundle = make_bundle(
            0x22,
            "validator-14",
            "mainnet",
            CryptoProfile::ClassicEd25519,
        );

        bundle.bundle_fingerprint = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string();

        let result = bundle.validate();
        assert_eq!(result, Err(NodeKeyBundleError::BundleFingerprintMismatch));
    }
}
