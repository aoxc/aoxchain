/// AOXC governance-controlled family namespace root.
///
/// Security rationale:
/// This value is a family-level namespace anchor rather than a specific network
/// discriminator. It must remain stable across AOXC network lines unless
/// governance explicitly authorizes a family reset.
pub const AOXC_FAMILY_ID: u32 = 2626;

/// Canonical AOXC network identifier prefix.
pub const AOXC_NETWORK_ID_PREFIX: &str = "aoxc";

/// Maximum allowed governance serial ordinal.
///
/// Example:
/// - `2626-001`
/// - `2626-999`
pub const MAX_NETWORK_SERIAL_ORDINAL: u16 = 999;

/// Maximum allowed instance ordinal inside a network-class segment.
///
/// Numeric chain-id layout:
/// `FFFFCCNNNN`
pub const MAX_NETWORK_INSTANCE_ORDINAL: u32 = 9999;

/// Canonical genesis encoding version.
///
/// Governance note:
/// This value must be incremented whenever canonical fingerprint framing or
/// identity preimage layout changes in a backward-incompatible manner.
pub const GENESIS_ENCODING_VERSION: u8 = 1;

/// Canonical default genesis timestamp.
///
/// Production rationale:
/// A deterministic default avoids runtime-dependent fingerprint drift. Real
/// production genesis artifacts may override this explicitly.
pub const DEFAULT_GENESIS_TIMESTAMP_UNIX: u64 = 1_700_000_000;

/// Domain separator for identity fingerprint derivation.
const DOMAIN_CHAIN_IDENTITY_FINGERPRINT: &[u8] = b"AOXC/GENESIS/CHAIN_IDENTITY/V1";

/// Domain separator for full genesis fingerprint derivation.
const DOMAIN_GENESIS_STATE_FINGERPRINT: &[u8] = b"AOXC/GENESIS/STATE/V1";

/// Maximum permitted chain-name length.
const MAX_CHAIN_NAME_LEN: usize = 128;

/// Maximum permitted protocol-version length.
const MAX_PROTOCOL_VERSION_LEN: usize = 64;

/// Maximum permitted genesis-notes length.
const MAX_GENESIS_NOTES_LEN: usize = 4096;

/// Maximum permitted endpoint length.
const MAX_ENDPOINT_LEN: usize = 512;

/// Maximum permitted identifier length for validator, account, node, and seal ids.
const MAX_IDENTIFIER_LEN: usize = 128;

/// Maximum permitted algorithm-name length.
const MAX_ALGORITHM_NAME_LEN: usize = 64;

/// Canonical network class.
///
/// The class code is embedded into the numeric `chain_id`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkClass {
    /// Canonical public mainnet line.
    PublicMainnet,

    /// Canonical public testnet line.
    PublicTestnet,

    /// Development or experimental network line.
    Devnet,

    /// Pre-production validation or staging network line.
    Validation,

    /// Sovereign private Layer 1 deployment line.
    SovereignPrivate,

    /// Consortium or federated enterprise network line.
    Consortium,

    /// Regulated or supervised private deployment line.
    RegulatedPrivate,
}

impl NetworkClass {
    /// Returns the canonical two-digit class code used in numeric `chain_id`
    /// derivation.
    ///
    /// Numeric chain identifier layout:
    /// `FFFFCCNNNN`
    #[must_use]
    pub const fn class_code(self) -> u16 {
        match self {
            Self::PublicMainnet => 0,
            Self::PublicTestnet => 1,
            Self::Devnet => 2,
            Self::Validation => 3,
            Self::SovereignPrivate => 10,
            Self::Consortium => 20,
            Self::RegulatedPrivate => 30,
        }
    }

    /// Returns the canonical machine-readable slug.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::PublicMainnet => "mainnet",
            Self::PublicTestnet => "testnet",
            Self::Devnet => "devnet",
            Self::Validation => "validation",
            Self::SovereignPrivate => "sovereign-private",
            Self::Consortium => "consortium",
            Self::RegulatedPrivate => "regulated-private",
        }
    }

    /// Returns whether the network class is public-facing.
    #[must_use]
    pub const fn is_public(self) -> bool {
        matches!(self, Self::PublicMainnet | Self::PublicTestnet)
    }
}

impl fmt::Display for NetworkClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}
