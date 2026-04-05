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

