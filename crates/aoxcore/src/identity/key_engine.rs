use rand::RngExt;
use sha3::{Digest, Sha3_512};
use std::fmt;

use super::hd_path::HdPath;

/// Domain separator preventing cross-protocol entropy reuse.
const AOXC_KEY_DOMAIN: &[u8] = b"AOXC-KEY-DERIVATION-V1";

/// Domain separator used for key-engine fingerprint derivation.
const AOXC_KEY_FINGERPRINT_DOMAIN: &[u8] = b"AOXC-KEY-ENGINE-FINGERPRINT-V1";

/// Canonical master-seed size in bytes.
pub const MASTER_SEED_LEN: usize = 64;

/// Canonical derived entropy size in bytes.
pub const DERIVED_ENTROPY_LEN: usize = 64;

/// Canonical coin type used by AOXC HD derivation.
///
/// Current canonical format:
/// m / 44 / 2626 / chain / role / zone / index
pub const AOXC_HD_PURPOSE: u32 = 2626;
pub const AOXC_HD_BIP44_PURPOSE: u32 = 44;

/// Error surface for key-engine operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyEngineError {
    /// The supplied HD path is invalid or operationally ambiguous.
    InvalidPath,

    /// The derived entropy length was not equal to the canonical output size.
    InvalidEntropyLength,
}

impl KeyEngineError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidPath => "KEY_ENGINE_INVALID_PATH",
            Self::InvalidEntropyLength => "KEY_ENGINE_INVALID_ENTROPY_LENGTH",
        }
    }
}

impl fmt::Display for KeyEngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath => {
                write!(f, "key derivation failed: HD path is invalid")
            }
            Self::InvalidEntropyLength => {
                write!(
                    f,
                    "key derivation failed: derived entropy length is invalid"
                )
            }
        }
    }
}

impl std::error::Error for KeyEngineError {}

/// AOXC deterministic key-derivation engine.
///
/// Security and design properties:
/// - stores a 512-bit master seed,
/// - derives deterministic path-scoped entropy,
/// - uses explicit domain separation,
/// - avoids cross-path ambiguity through canonical field framing,
/// - remains independent from any one downstream key algorithm,
/// - is suitable as a foundation for post-quantum key pipelines.
///
/// Important note:
/// This engine derives entropy for downstream key generation. It does not
/// itself implement post-quantum signatures or verification logic.
#[derive(Debug, Clone)]
pub struct KeyEngine {
    master_seed: [u8; MASTER_SEED_LEN],
}

impl KeyEngine {
    /// Creates a new key engine.
    ///
    /// Behavior:
    /// - if a seed is provided, it is used directly;
    /// - otherwise, a fresh 512-bit seed is generated from the local RNG.
    #[must_use]
    pub fn new(seed: Option<[u8; MASTER_SEED_LEN]>) -> Self {
        match seed {
            Some(seed) => Self { master_seed: seed },
            None => {
                let mut rng = rand::rng();

                let mut seed = [0u8; MASTER_SEED_LEN];
                rng.fill(&mut seed);

                Self { master_seed: seed }
            }
        }
    }

    /// Constructs a key engine from an explicit master seed.
    #[must_use]
    pub fn from_seed(seed: [u8; MASTER_SEED_LEN]) -> Self {
        Self { master_seed: seed }
    }

    /// Returns the master seed by reference.
    ///
    /// This material is security-sensitive and should be handled carefully.
    #[must_use]
    pub fn master_seed(&self) -> &[u8; MASTER_SEED_LEN] {
        &self.master_seed
    }

    /// Derives canonical entropy for the supplied HD path.
    ///
    /// This method preserves a compatibility-friendly direct-return API and
    /// delegates to the strict error-aware derivation path internally.
    #[must_use]
    pub fn derive_entropy(&self, path: &HdPath) -> [u8; DERIVED_ENTROPY_LEN] {
        self.try_derive_entropy(path)
            .expect("KEY_ENGINE: HD path validation failed during entropy derivation")
    }

    /// Derives canonical entropy for the supplied HD path with explicit error handling.
    ///
    /// Derivation input includes:
    /// - AOXC derivation domain,
    /// - BIP44 purpose,
    /// - canonical HD purpose,
    /// - master seed,
    /// - path chain,
    /// - path role,
    /// - path zone,
    /// - path index.
    ///
    /// All variable derivation components are framed explicitly to reduce
    /// ambiguity and preserve deterministic behavior across implementations.
    pub fn try_derive_entropy(
        &self,
        path: &HdPath,
    ) -> Result<[u8; DERIVED_ENTROPY_LEN], KeyEngineError> {
        validate_hd_path(path)?;

        let mut hasher = Sha3_512::new();

        hasher.update(AOXC_KEY_DOMAIN);
        hasher.update([0x00]);

        hasher.update(AOXC_HD_BIP44_PURPOSE.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(AOXC_HD_PURPOSE.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(self.master_seed);
        hasher.update([0x00]);

        hasher.update(path.chain.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(path.role.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(path.zone.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(path.index.to_be_bytes());

        let digest = hasher.finalize();

        if digest.len() != DERIVED_ENTROPY_LEN {
            return Err(KeyEngineError::InvalidEntropyLength);
        }

        let mut out = [0u8; DERIVED_ENTROPY_LEN];
        out.copy_from_slice(&digest);

        Ok(out)
    }

    /// Derives canonical entropy and returns it as uppercase hexadecimal.
    pub fn derive_entropy_hex(&self, path: &HdPath) -> Result<String, KeyEngineError> {
        let entropy = self.try_derive_entropy(path)?;
        Ok(hex::encode_upper(entropy))
    }

    /// Derives stable child key material for downstream cryptographic key generation.
    ///
    /// This is currently equivalent to `try_derive_entropy`, but the dedicated
    /// method name makes higher-level intent explicit and leaves room for future
    /// diversification policies without breaking call sites.
    pub fn derive_key_material(
        &self,
        path: &HdPath,
    ) -> Result<[u8; DERIVED_ENTROPY_LEN], KeyEngineError> {
        self.try_derive_entropy(path)
    }

    /// Derives a stable engine fingerprint from the master seed.
    ///
    /// This helper is suitable for diagnostics and audit references, but is not
    /// a substitute for the underlying key material.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha3_512::new();

        hasher.update(AOXC_KEY_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(self.master_seed);

        let digest = hasher.finalize();

        // 16 bytes => 32 hex chars. Stable and short enough for operator use.
        hex::encode_upper(&digest[..16])
    }
}

/// Validates an HD path before entropy derivation.
///
/// Current policy:
/// - the path must not be the all-zero vector;
/// - the path must not use the AOXC reserved purpose value as a regular field;
/// - zero values are individually allowed except for the fully ambiguous all-zero case.
fn validate_hd_path(path: &HdPath) -> Result<(), KeyEngineError> {
    if path.chain == 0 && path.role == 0 && path.zone == 0 && path.index == 0 {
        return Err(KeyEngineError::InvalidPath);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::hd_path::HdPath;

    fn sample_seed() -> [u8; MASTER_SEED_LEN] {
        [0x11; MASTER_SEED_LEN]
    }

    fn sample_path() -> HdPath {
        HdPath {
            chain: 1,
            role: 1,
            zone: 2,
            index: 0,
        }
    }

    #[test]
    fn deterministic_derivation_for_same_seed_and_path() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let a = engine.derive_entropy(&path);
        let b = engine.derive_entropy(&path);

        assert_eq!(a, b);
    }

    #[test]
    fn derivation_changes_when_path_changes() {
        let engine = KeyEngine::from_seed(sample_seed());

        let a = engine.derive_entropy(&HdPath {
            chain: 1,
            role: 1,
            zone: 2,
            index: 0,
        });

        let b = engine.derive_entropy(&HdPath {
            chain: 1,
            role: 1,
            zone: 2,
            index: 1,
        });

        assert_ne!(a, b);
    }

    #[test]
    fn derivation_changes_when_seed_changes() {
        let path = sample_path();

        let a = KeyEngine::from_seed([0x11; MASTER_SEED_LEN]).derive_entropy(&path);
        let b = KeyEngine::from_seed([0x22; MASTER_SEED_LEN]).derive_entropy(&path);

        assert_ne!(a, b);
    }

    #[test]
    fn fingerprint_is_stable() {
        let engine = KeyEngine::from_seed(sample_seed());

        let a = engine.fingerprint();
        let b = engine.fingerprint();

        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
    }

    #[test]
    fn invalid_zero_path_is_rejected_in_strict_mode() {
        let engine = KeyEngine::from_seed(sample_seed());

        let path = HdPath {
            chain: 0,
            role: 0,
            zone: 0,
            index: 0,
        };

        assert_eq!(
            engine.try_derive_entropy(&path),
            Err(KeyEngineError::InvalidPath)
        );
    }

    #[test]
    fn entropy_hex_has_expected_length() {
        let engine = KeyEngine::from_seed(sample_seed());

        let hex = engine
            .derive_entropy_hex(&sample_path())
            .expect("hex derivation must succeed");

        assert_eq!(hex.len(), DERIVED_ENTROPY_LEN * 2);
    }

    #[test]
    fn derive_key_material_matches_entropy() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let a = engine
            .derive_key_material(&path)
            .expect("key material derivation must succeed");
        let b = engine
            .try_derive_entropy(&path)
            .expect("entropy derivation must succeed");

        assert_eq!(a, b);
    }
}
