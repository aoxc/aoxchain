// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use rand::RngExt;
use sha3::{Digest, Sha3_256, Sha3_512};
use std::fmt;

use super::hd_path::HdPath;

/// Domain separator preventing cross-protocol entropy reuse.
const AOXC_KEY_DOMAIN: &[u8] = b"AOXC/IDENTITY/KEY_ENGINE/V1";

/// Domain separator used for key-engine fingerprint derivation.
const AOXC_KEY_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/KEY_ENGINE/FINGERPRINT/V1";

/// Domain separator used for role-scoped seed derivation.
const AOXC_ROLE_SEED_DOMAIN: &[u8] = b"AOXC/IDENTITY/KEY_ENGINE/ROLE_SEED/V1";

/// Canonical master-seed size in bytes.
pub const MASTER_SEED_LEN: usize = 64;

/// Canonical derived entropy size in bytes.
pub const DERIVED_ENTROPY_LEN: usize = 64;

/// Canonical role-seed length in bytes.
///
/// This helper output is intended for downstream algorithms that prefer
/// compact deterministic seed material.
pub const ROLE_SEED_LEN: usize = 32;

/// Canonical coin type used by AOXC HD derivation.
///
/// Current canonical format:
/// m / 44 / 2626 / chain / role / zone / index
pub const AOXC_HD_PURPOSE: u32 = 2626;
pub const AOXC_HD_BIP44_PURPOSE: u32 = 44;

/// Maximum accepted canonical HD path component.
///
/// AOXC canonical path derivation currently assumes unhardened variable
/// components only. Hardened derivation remains a downstream projection concern.
pub const MAX_CANONICAL_HD_COMPONENT: u32 = 0x7FFF_FFFF;

/// Maximum accepted role-label length.
///
/// This bound is intentionally conservative while remaining flexible enough
/// for operational and protocol-facing role labels.
pub const MAX_ROLE_LABEL_LEN: usize = 64;

/// Error surface for key-engine operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyEngineError {
    /// The supplied HD path is invalid or operationally ambiguous.
    InvalidPath,

    /// The derived entropy length was not equal to the canonical output size.
    InvalidEntropyLength,

    /// The supplied role label was empty.
    EmptyRoleLabel,

    /// The supplied role label was present but not canonical.
    InvalidRoleLabel,
}

impl KeyEngineError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidPath => "KEY_ENGINE_INVALID_PATH",
            Self::InvalidEntropyLength => "KEY_ENGINE_INVALID_ENTROPY_LENGTH",
            Self::EmptyRoleLabel => "KEY_ENGINE_EMPTY_ROLE_LABEL",
            Self::InvalidRoleLabel => "KEY_ENGINE_INVALID_ROLE_LABEL",
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
            Self::EmptyRoleLabel => {
                write!(f, "key derivation failed: role label must not be empty")
            }
            Self::InvalidRoleLabel => {
                write!(f, "key derivation failed: role label is not canonical")
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

impl Drop for KeyEngine {
    fn drop(&mut self) {
        self.master_seed.fill(0);
    }
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
    pub fn derive_entropy(
        &self,
        path: &HdPath,
    ) -> Result<[u8; DERIVED_ENTROPY_LEN], KeyEngineError> {
        self.try_derive_entropy(path)
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

    /// Derives compact role-scoped seed material from an existing canonical
    /// path-scoped entropy output.
    ///
    /// This helper is intended for downstream algorithms that prefer a compact
    /// deterministic 32-byte seed rather than a 64-byte entropy surface.
    pub fn derive_role_seed(
        &self,
        path: &HdPath,
        role_label: &str,
    ) -> Result<[u8; ROLE_SEED_LEN], KeyEngineError> {
        let material = self.derive_key_material(path)?;
        derive_role_seed_from_material(&material, role_label)
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

/// Derives a compact role-scoped 32-byte seed from AOXC key material.
///
/// Canonical role-label policy:
/// - label must not be blank,
/// - surrounding whitespace is rejected rather than normalized,
/// - internal whitespace is forbidden,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted,
/// - length must remain bounded.
pub fn derive_role_seed_from_material(
    material: &[u8; DERIVED_ENTROPY_LEN],
    role_label: &str,
) -> Result<[u8; ROLE_SEED_LEN], KeyEngineError> {
    validate_role_label(role_label)?;

    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_ROLE_SEED_DOMAIN);
    hasher.update([0x00]);
    hasher.update(role_label.as_bytes());
    hasher.update([0x00]);
    hasher.update(material);

    let digest = hasher.finalize();

    let mut out = [0u8; ROLE_SEED_LEN];
    out.copy_from_slice(&digest[..ROLE_SEED_LEN]);

    Ok(out)
}

/// Validates an HD path before entropy derivation.
///
/// Current policy:
/// - the path must not be the all-zero vector;
/// - every variable component must remain in the canonical unhardened range.
fn validate_hd_path(path: &HdPath) -> Result<(), KeyEngineError> {
    if path.chain == 0 && path.role == 0 && path.zone == 0 && path.index == 0 {
        return Err(KeyEngineError::InvalidPath);
    }

    if path.chain > MAX_CANONICAL_HD_COMPONENT
        || path.role > MAX_CANONICAL_HD_COMPONENT
        || path.zone > MAX_CANONICAL_HD_COMPONENT
        || path.index > MAX_CANONICAL_HD_COMPONENT
    {
        return Err(KeyEngineError::InvalidPath);
    }

    Ok(())
}

/// Validates a canonical role label.
fn validate_role_label(role_label: &str) -> Result<(), KeyEngineError> {
    if role_label.is_empty() || role_label.trim().is_empty() {
        return Err(KeyEngineError::EmptyRoleLabel);
    }

    if role_label != role_label.trim() {
        return Err(KeyEngineError::InvalidRoleLabel);
    }

    if role_label.len() > MAX_ROLE_LABEL_LEN {
        return Err(KeyEngineError::InvalidRoleLabel);
    }

    if role_label.chars().any(|ch| ch.is_ascii_whitespace()) {
        return Err(KeyEngineError::InvalidRoleLabel);
    }

    if !role_label
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(KeyEngineError::InvalidRoleLabel);
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

        let a = engine
            .derive_entropy(&path)
            .expect("entropy derivation must succeed");
        let b = engine
            .derive_entropy(&path)
            .expect("entropy derivation must succeed");

        assert_eq!(a, b);
    }

    #[test]
    fn derivation_changes_when_path_changes() {
        let engine = KeyEngine::from_seed(sample_seed());

        let a = engine
            .derive_entropy(&HdPath {
                chain: 1,
                role: 1,
                zone: 2,
                index: 0,
            })
            .expect("entropy derivation must succeed");

        let b = engine
            .derive_entropy(&HdPath {
                chain: 1,
                role: 1,
                zone: 2,
                index: 1,
            })
            .expect("entropy derivation must succeed");

        assert_ne!(a, b);
    }

    #[test]
    fn derivation_changes_when_seed_changes() {
        let path = sample_path();

        let a = KeyEngine::from_seed([0x11; MASTER_SEED_LEN])
            .derive_entropy(&path)
            .expect("entropy derivation must succeed");
        let b = KeyEngine::from_seed([0x22; MASTER_SEED_LEN])
            .derive_entropy(&path)
            .expect("entropy derivation must succeed");

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
    fn out_of_range_component_is_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());

        let path = HdPath {
            chain: MAX_CANONICAL_HD_COMPONENT + 1,
            role: 1,
            zone: 1,
            index: 1,
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

    #[test]
    fn role_seed_derivation_is_deterministic() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let a = engine
            .derive_role_seed(&path, "consensus")
            .expect("role seed derivation must succeed");
        let b = engine
            .derive_role_seed(&path, "consensus")
            .expect("role seed derivation must succeed");

        assert_eq!(a, b);
    }

    #[test]
    fn role_seed_derivation_changes_by_label() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let a = engine
            .derive_role_seed(&path, "consensus")
            .expect("role seed derivation must succeed");
        let b = engine
            .derive_role_seed(&path, "transport")
            .expect("role seed derivation must succeed");

        assert_ne!(a, b);
    }

    #[test]
    fn empty_role_label_is_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let result = engine.derive_role_seed(&path, "");
        assert_eq!(result, Err(KeyEngineError::EmptyRoleLabel));
    }

    #[test]
    fn whitespace_only_role_label_is_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let result = engine.derive_role_seed(&path, "   ");
        assert_eq!(result, Err(KeyEngineError::EmptyRoleLabel));
    }

    #[test]
    fn surrounding_whitespace_in_role_label_is_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let result = engine.derive_role_seed(&path, " consensus ");
        assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
    }

    #[test]
    fn internal_whitespace_in_role_label_is_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let result = engine.derive_role_seed(&path, "consensus role");
        assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
    }

    #[test]
    fn invalid_characters_in_role_label_are_rejected() {
        let engine = KeyEngine::from_seed(sample_seed());
        let path = sample_path();

        let result = engine.derive_role_seed(&path, "consensus!");
        assert_eq!(result, Err(KeyEngineError::InvalidRoleLabel));
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(KeyEngineError::InvalidPath.code(), "KEY_ENGINE_INVALID_PATH");
        assert_eq!(
            KeyEngineError::InvalidEntropyLength.code(),
            "KEY_ENGINE_INVALID_ENTROPY_LENGTH"
        );
        assert_eq!(
            KeyEngineError::EmptyRoleLabel.code(),
            "KEY_ENGINE_EMPTY_ROLE_LABEL"
        );
        assert_eq!(
            KeyEngineError::InvalidRoleLabel.code(),
            "KEY_ENGINE_INVALID_ROLE_LABEL"
        );
    }
}
