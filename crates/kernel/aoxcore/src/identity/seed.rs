// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC root-seed lifecycle primitives.
//!
//! This module is intentionally separated from `key_engine.rs`.
//!
//! Responsibility split:
//! - `seed.rs`      => root secret generation, restore, fingerprinting, custody handoff,
//! - `key_engine.rs` => deterministic derivation from an internal master seed,
//! - `keyfile.rs`   => encrypted persistence / custody,
//! - `key_bundle.rs` => public operational key packaging.
//!
//! Security posture:
//! - root seed generation uses operating-system / kernel randomness only,
//! - the recovery seed and internal master seed are separated,
//! - master seed derivation is domain-separated and seed-kind bound,
//! - secret buffers are zeroed on drop,
//! - encrypted persistence is delegated to `keyfile.rs`.
//!
//! Important note:
//! This module is designed to be the secure seed core. Human-readable recovery
//! formats such as 24-word mnemonics should be implemented in a dedicated
//! `mnemonic.rs` layer rather than embedded here.

use getrandom::fill as getrandom_fill;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Sha3_512};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use super::key_engine::{KeyEngine, MASTER_SEED_LEN};
use super::keyfile::{KeyfileEnvelope, KeyfileError, encrypt_key_to_envelope};

/// Current AOXC seed schema version.
pub const AOXC_SEED_VERSION: u8 = 1;

/// Canonical recovery-seed length in bytes.
///
/// Security rationale:
/// - 32 bytes = 256 bits of entropy,
/// - suitable as a user-visible root backup secret,
/// - strong enough for post-quantum-aware custody assumptions.
pub const RECOVERY_SEED_LEN: usize = 32;

/// Canonical operator-facing seed fingerprint length in bytes.
pub const SEED_FINGERPRINT_LEN: usize = 8;

/// Maximum accepted additional-entropy length.
///
/// Additional entropy is optional and is mixed into the final recoverable seed.
/// The bound prevents abuse of the generation surface with unbounded input.
pub const MAX_ADDITIONAL_ENTROPY_LEN: usize = 4096;

/// Domain separator for recovery-seed generation.
const AOXC_SEED_GENERATION_DOMAIN: &[u8] = b"AOXC/IDENTITY/SEED/GENERATION/V1";

/// Domain separator for master-seed derivation.
const AOXC_MASTER_SEED_DERIVATION_DOMAIN: &[u8] = b"AOXC/IDENTITY/SEED/MASTER/V1";

/// Domain separator for seed fingerprint derivation.
const AOXC_SEED_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/SEED/FINGERPRINT/V1";

/// Defines the custody domain of a root seed.
///
/// Design rationale:
/// different trust zones should not reuse the same root seed.
///
/// Recommended model:
/// - one root seed per wallet,
/// - one root seed per node,
/// - one root seed per treasury / CA domain,
/// - not one seed per address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SeedKind {
    WalletRoot,
    NodeRoot,
    TreasuryRoot,
    CaRoot,
}

impl SeedKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WalletRoot => "wallet-root",
            Self::NodeRoot => "node-root",
            Self::TreasuryRoot => "treasury-root",
            Self::CaRoot => "ca-root",
        }
    }
}

impl fmt::Display for SeedKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Safe public metadata that may be logged or serialized.
///
/// This structure intentionally excludes all secret bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedMetadata {
    pub version: u8,
    pub kind: SeedKind,
    pub materialized_at: u64,
    pub fingerprint: String,
}

/// Secret seed object returned by the AOXC seed layer.
///
/// Operational model:
/// - `recovery_seed` is the user-backup root secret source,
/// - `master_seed` is the internal expanded seed consumed by `KeyEngine`,
/// - callers should avoid cloning or logging secret material.
pub struct GeneratedSeed {
    metadata: SeedMetadata,
    recovery_seed: [u8; RECOVERY_SEED_LEN],
    master_seed: [u8; MASTER_SEED_LEN],
}

impl fmt::Debug for GeneratedSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedSeed")
            .field("metadata", &self.metadata)
            .field("recovery_seed_len", &RECOVERY_SEED_LEN)
            .field("master_seed_len", &MASTER_SEED_LEN)
            .finish()
    }
}

impl Drop for GeneratedSeed {
    fn drop(&mut self) {
        self.recovery_seed.fill(0);
        self.master_seed.fill(0);
    }
}

/// Canonical seed error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeedError {
    EntropyUnavailable,
    InvalidAdditionalEntropy,
    InvalidRecoverySeedLength,
    InvalidRecoverySeedHex,
    InvalidMaterializedAt,
    TimeError,
    Keyfile(KeyfileError),
}

impl SeedError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EntropyUnavailable => "SEED_ENTROPY_UNAVAILABLE",
            Self::InvalidAdditionalEntropy => "SEED_INVALID_ADDITIONAL_ENTROPY",
            Self::InvalidRecoverySeedLength => "SEED_INVALID_RECOVERY_SEED_LENGTH",
            Self::InvalidRecoverySeedHex => "SEED_INVALID_RECOVERY_SEED_HEX",
            Self::InvalidMaterializedAt => "SEED_INVALID_MATERIALIZED_AT",
            Self::TimeError => "SEED_TIME_ERROR",
            Self::Keyfile(_) => "SEED_KEYFILE_ERROR",
        }
    }
}

impl fmt::Display for SeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EntropyUnavailable => {
                write!(
                    f,
                    "seed generation failed: operating-system entropy is unavailable"
                )
            }
            Self::InvalidAdditionalEntropy => {
                write!(
                    f,
                    "seed generation failed: additional entropy input is invalid"
                )
            }
            Self::InvalidRecoverySeedLength => {
                write!(f, "seed restore failed: recovery seed length is invalid")
            }
            Self::InvalidRecoverySeedHex => {
                write!(f, "seed restore failed: recovery seed hex is invalid")
            }
            Self::InvalidMaterializedAt => {
                write!(f, "seed lifecycle failed: materialized_at is invalid")
            }
            Self::TimeError => {
                write!(f, "seed lifecycle failed: system time is invalid")
            }
            Self::Keyfile(error) => {
                write!(f, "seed custody failed: {}", error)
            }
        }
    }
}

impl std::error::Error for SeedError {}

impl From<KeyfileError> for SeedError {
    fn from(value: KeyfileError) -> Self {
        Self::Keyfile(value)
    }
}

impl GeneratedSeed {
    /// Generates a new AOXC seed from OS / kernel randomness only.
    ///
    /// This is the preferred default constructor for wallet-first or node-first
    /// root-seed generation.
    pub fn generate(kind: SeedKind) -> Result<Self, SeedError> {
        generate_seed(kind)
    }

    /// Generates a new AOXC seed and mixes optional caller-supplied entropy.
    ///
    /// Security note:
    /// - OS CSPRNG remains the primary trust source,
    /// - caller-supplied entropy is additive,
    /// - the final displayed recovery seed already contains the mixed result,
    ///   so recovery remains possible from the emitted seed alone.
    pub fn generate_with_additional_entropy(
        kind: SeedKind,
        additional_entropy: &[u8],
    ) -> Result<Self, SeedError> {
        generate_seed_with_additional_entropy(kind, additional_entropy)
    }

    /// Restores a seed object from an existing recovery seed.
    pub fn from_recovery_seed(
        kind: SeedKind,
        recovery_seed: [u8; RECOVERY_SEED_LEN],
    ) -> Result<Self, SeedError> {
        let materialized_at = current_time()?;
        Self::from_recovery_seed_at(kind, recovery_seed, materialized_at)
    }

    /// Restores a seed object from a recovery-seed byte slice.
    pub fn from_recovery_seed_bytes(
        kind: SeedKind,
        recovery_seed: &[u8],
    ) -> Result<Self, SeedError> {
        if recovery_seed.len() != RECOVERY_SEED_LEN {
            return Err(SeedError::InvalidRecoverySeedLength);
        }

        let mut out = [0u8; RECOVERY_SEED_LEN];
        out.copy_from_slice(recovery_seed);

        Self::from_recovery_seed(kind, out)
    }

    /// Restores a seed object from a recovery-seed hex string.
    ///
    /// Expected format:
    /// - uppercase or lowercase hexadecimal,
    /// - exactly `RECOVERY_SEED_LEN * 2` characters.
    pub fn from_recovery_seed_hex(kind: SeedKind, encoded: &str) -> Result<Self, SeedError> {
        if encoded.is_empty() || encoded.trim().is_empty() || encoded != encoded.trim() {
            return Err(SeedError::InvalidRecoverySeedHex);
        }

        let decoded = hex::decode(encoded).map_err(|_| SeedError::InvalidRecoverySeedHex)?;
        Self::from_recovery_seed_bytes(kind, &decoded)
    }

    /// Restores a seed object from an existing recovery seed with an explicit timestamp.
    ///
    /// This helper is suitable for deterministic tests and replayable workflows.
    pub fn from_recovery_seed_at(
        kind: SeedKind,
        recovery_seed: [u8; RECOVERY_SEED_LEN],
        materialized_at: u64,
    ) -> Result<Self, SeedError> {
        if materialized_at == 0 {
            return Err(SeedError::InvalidMaterializedAt);
        }

        let master_seed = derive_master_seed(kind, &recovery_seed);
        let fingerprint = compute_seed_fingerprint(kind, &master_seed);

        Ok(Self {
            metadata: SeedMetadata {
                version: AOXC_SEED_VERSION,
                kind,
                materialized_at,
                fingerprint,
            },
            recovery_seed,
            master_seed,
        })
    }

    /// Returns safe public metadata for the seed object.
    #[must_use]
    pub fn metadata(&self) -> &SeedMetadata {
        &self.metadata
    }

    /// Returns the recovery seed bytes.
    ///
    /// Security note:
    /// this is the user-backup root secret and must be treated as highly sensitive.
    #[must_use]
    pub fn recovery_seed(&self) -> &[u8; RECOVERY_SEED_LEN] {
        &self.recovery_seed
    }

    /// Returns the internal master seed bytes.
    ///
    /// Security note:
    /// this material should remain internal to AOXC derivation and custody layers.
    #[must_use]
    pub fn master_seed(&self) -> &[u8; MASTER_SEED_LEN] {
        &self.master_seed
    }

    /// Returns the recovery seed encoded as uppercase hexadecimal.
    ///
    /// This is a suitable intermediate export form for:
    /// - QR encoding,
    /// - air-gapped backup transfer,
    /// - future mnemonic conversion layers.
    #[must_use]
    pub fn recovery_seed_hex(&self) -> String {
        hex::encode_upper(self.recovery_seed)
    }

    /// Returns the master seed encoded as uppercase hexadecimal.
    ///
    /// This helper is operationally dangerous and should generally not be shown
    /// to end users or written to logs.
    #[must_use]
    pub fn master_seed_hex(&self) -> String {
        hex::encode_upper(self.master_seed)
    }

    /// Builds a `KeyEngine` from the generated master seed.
    #[must_use]
    pub fn to_key_engine(&self) -> KeyEngine {
        KeyEngine::from_seed(self.master_seed)
    }

    /// Encrypts the internal master seed into a keyfile envelope.
    ///
    /// This is the preferred persistence path for AOXC nodes and wallets that
    /// do not want plaintext seed material at rest.
    pub fn encrypt_master_seed(&self, password: &str) -> Result<KeyfileEnvelope, SeedError> {
        encrypt_key_to_envelope(&self.master_seed, password).map_err(SeedError::from)
    }
}

/// Generates a new AOXC seed from OS / kernel randomness only.
pub fn generate_seed(kind: SeedKind) -> Result<GeneratedSeed, SeedError> {
    let materialized_at = current_time()?;

    let mut kernel_entropy = [0u8; RECOVERY_SEED_LEN];
    getrandom_fill(&mut kernel_entropy).map_err(|_| SeedError::EntropyUnavailable)?;

    GeneratedSeed::from_recovery_seed_at(kind, kernel_entropy, materialized_at)
}

/// Generates a new AOXC seed and mixes optional caller-supplied entropy.
///
/// Final recoverable seed policy:
/// `SHA3-256(domain || kind || kernel_entropy || digest(additional_entropy))`
pub fn generate_seed_with_additional_entropy(
    kind: SeedKind,
    additional_entropy: &[u8],
) -> Result<GeneratedSeed, SeedError> {
    if additional_entropy.is_empty() || additional_entropy.len() > MAX_ADDITIONAL_ENTROPY_LEN {
        return Err(SeedError::InvalidAdditionalEntropy);
    }

    let materialized_at = current_time()?;

    let mut kernel_entropy = [0u8; RECOVERY_SEED_LEN];
    getrandom_fill(&mut kernel_entropy).map_err(|_| SeedError::EntropyUnavailable)?;

    let additional_digest = digest_additional_entropy(additional_entropy);

    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_SEED_GENERATION_DOMAIN);
    hasher.update([0x00]);
    hasher.update(kind.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(kernel_entropy);
    hasher.update([0x00]);
    hasher.update(additional_digest);

    let digest = hasher.finalize();

    let mut recovery_seed = [0u8; RECOVERY_SEED_LEN];
    recovery_seed.copy_from_slice(&digest[..RECOVERY_SEED_LEN]);

    GeneratedSeed::from_recovery_seed_at(kind, recovery_seed, materialized_at)
}

/// Derives the AOXC internal master seed from the recovery seed.
///
/// Design rationale:
/// - recovery seed is the user-backup root secret,
/// - master seed is an expanded internal seed surface,
/// - derivation is domain-separated and seed-kind bound.
fn derive_master_seed(
    kind: SeedKind,
    recovery_seed: &[u8; RECOVERY_SEED_LEN],
) -> [u8; MASTER_SEED_LEN] {
    let mut hasher = Sha3_512::new();
    hasher.update(AOXC_MASTER_SEED_DERIVATION_DOMAIN);
    hasher.update([0x00]);
    hasher.update(kind.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(recovery_seed);

    let digest = hasher.finalize();

    let mut out = [0u8; MASTER_SEED_LEN];
    out.copy_from_slice(&digest[..MASTER_SEED_LEN]);
    out
}

/// Computes a short fingerprint for operator-facing diagnostics.
///
/// This is safe for:
/// - audit notes,
/// - support workflows,
/// - operator UI reference.
///
/// It is not a substitute for the underlying seed material.
fn compute_seed_fingerprint(kind: SeedKind, master_seed: &[u8; MASTER_SEED_LEN]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_SEED_FINGERPRINT_DOMAIN);
    hasher.update([0x00]);
    hasher.update(kind.as_str().as_bytes());
    hasher.update([0x00]);
    hasher.update(master_seed);

    let digest = hasher.finalize();
    hex::encode_upper(&digest[..SEED_FINGERPRINT_LEN])
}

/// Digests caller-supplied additive entropy into a bounded fixed-length form.
fn digest_additional_entropy(additional_entropy: &[u8]) -> [u8; RECOVERY_SEED_LEN] {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_SEED_GENERATION_DOMAIN);
    hasher.update([0x00]);
    hasher.update(b"ADDITIONAL_ENTROPY");
    hasher.update([0x00]);
    hasher.update(additional_entropy);

    let digest = hasher.finalize();

    let mut out = [0u8; RECOVERY_SEED_LEN];
    out.copy_from_slice(&digest[..RECOVERY_SEED_LEN]);
    out
}

/// Returns the current UNIX timestamp in seconds.
fn current_time() -> Result<u64, SeedError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| SeedError::TimeError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_seed_has_expected_lengths() {
        let seed = generate_seed(SeedKind::WalletRoot).expect("seed generation must succeed");

        assert_eq!(seed.recovery_seed().len(), RECOVERY_SEED_LEN);
        assert_eq!(seed.master_seed().len(), MASTER_SEED_LEN);
        assert_eq!(seed.recovery_seed_hex().len(), RECOVERY_SEED_LEN * 2);
        assert_eq!(seed.metadata().fingerprint.len(), SEED_FINGERPRINT_LEN * 2);
    }

    #[test]
    fn recovery_seed_restore_is_deterministic() {
        let original = generate_seed(SeedKind::WalletRoot).expect("seed generation must succeed");
        let restored =
            GeneratedSeed::from_recovery_seed(SeedKind::WalletRoot, *original.recovery_seed())
                .expect("seed restore must succeed");

        assert_eq!(original.recovery_seed(), restored.recovery_seed());
        assert_eq!(original.master_seed(), restored.master_seed());
        assert_eq!(
            original.metadata().fingerprint,
            restored.metadata().fingerprint
        );
    }

    #[test]
    fn recovery_seed_hex_restore_is_deterministic() {
        let original = generate_seed(SeedKind::WalletRoot).expect("seed generation must succeed");
        let encoded = original.recovery_seed_hex();

        let restored = GeneratedSeed::from_recovery_seed_hex(SeedKind::WalletRoot, &encoded)
            .expect("seed restore must succeed");

        assert_eq!(original.master_seed(), restored.master_seed());
    }

    #[test]
    fn different_seed_kinds_produce_different_master_seeds_from_same_recovery_seed() {
        let recovery_seed = [0x11; RECOVERY_SEED_LEN];

        let wallet = GeneratedSeed::from_recovery_seed_at(SeedKind::WalletRoot, recovery_seed, 1)
            .expect("wallet seed restore must succeed");
        let node = GeneratedSeed::from_recovery_seed_at(SeedKind::NodeRoot, recovery_seed, 1)
            .expect("node seed restore must succeed");

        assert_ne!(wallet.master_seed(), node.master_seed());
        assert_ne!(wallet.metadata().fingerprint, node.metadata().fingerprint);
    }

    #[test]
    fn additional_entropy_changes_the_output() {
        let a = generate_seed_with_additional_entropy(SeedKind::WalletRoot, b"dice-roll-entropy-A")
            .expect("seed generation must succeed");

        let b = generate_seed_with_additional_entropy(SeedKind::WalletRoot, b"dice-roll-entropy-B")
            .expect("seed generation must succeed");

        assert_ne!(a.recovery_seed(), b.recovery_seed());
        assert_ne!(a.master_seed(), b.master_seed());
    }

    #[test]
    fn invalid_additional_entropy_is_rejected() {
        let error = generate_seed_with_additional_entropy(SeedKind::WalletRoot, b"")
            .expect_err("generation must fail");

        assert_eq!(error, SeedError::InvalidAdditionalEntropy);
    }

    #[test]
    fn invalid_recovery_seed_hex_is_rejected() {
        let error = GeneratedSeed::from_recovery_seed_hex(SeedKind::WalletRoot, "ZZ_NOT_HEX")
            .expect_err("restore must fail");

        assert_eq!(error, SeedError::InvalidRecoverySeedHex);
    }

    #[test]
    fn key_engine_conversion_is_stable() {
        let seed = GeneratedSeed::from_recovery_seed_at(
            SeedKind::WalletRoot,
            [0x22; RECOVERY_SEED_LEN],
            1,
        )
        .expect("seed restore must succeed");

        let engine_a = seed.to_key_engine();
        let engine_b = seed.to_key_engine();

        assert_eq!(engine_a.fingerprint(), engine_b.fingerprint());
    }
}
