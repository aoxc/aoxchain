//! core/src/identity/actor_id.rs
//!
//! AOXC Actor ID Derivation and Validation Module.
//!
//! This module defines the canonical actor identifier format for AOXC identity
//! surfaces. It is designed for production use, deterministic derivation,
//! operator readability, registry indexing, and typo detection.
//!
//! Canonical format:
//!
//! AOXC-RRR-ZZ-SSSSSSSSSSSSSSSS-CC
//!
//! Example:
//!
//! AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9
//!
//! Where:
//! - AOXC : protocol prefix
//! - RRR  : normalized role code
//! - ZZ   : normalized zone code
//! - S... : deterministic serial derived from a domain-separated hash
//! - CC   : compact checksum for operator-facing typo detection
//!
//! Security and design objectives:
//! - Deterministic ID derivation from canonical input
//! - Explicit domain separation
//! - Strict parsing and validation
//! - Stable serialization format
//! - Human-readable and operationally searchable identifiers
//! - Reduced collision risk compared to short visible serials
//!
//! Important note:
//! This module provides identity derivation and format validation only.
//! It does not by itself prove cryptographic ownership of the referenced key.
//! Signature validation and certificate binding must be enforced elsewhere.

use sha3::{Digest, Sha3_256};
use std::fmt;

/// Canonical AOXC actor identifier prefix.
pub const AOXC_PREFIX: &str = "AOXC";

/// Canonical actor-id derivation domain separator.
///
/// This constant is embedded into the serial derivation hash to prevent
/// accidental cross-domain hash reuse.
const ACTOR_ID_DOMAIN: &[u8] = b"AOXC/IDENTITY/ACTOR_ID/V1";

/// Canonical actor-id checksum domain separator.
const ACTOR_ID_CHECKSUM_DOMAIN: &[u8] = b"AOXC/IDENTITY/ACTOR_ID/CHECKSUM/V1";

/// Canonical role width.
pub const ROLE_LEN: usize = 3;

/// Canonical zone width.
pub const ZONE_LEN: usize = 2;

/// Number of digest bytes used for the visible serial portion.
///
/// 8 bytes => 16 uppercase hexadecimal characters.
/// This is significantly safer than short 32-bit visible serials for
/// production-scale registries.
pub const SERIAL_BYTES: usize = 8;

/// Number of checksum symbols appended to the actor identifier.
pub const CHECKSUM_LEN: usize = 2;

/// Total number of dash-separated components in the canonical actor id.
const PART_COUNT: usize = 5;

/// Canonical visible serial length in hexadecimal characters.
const SERIAL_HEX_LEN: usize = SERIAL_BYTES * 2;

/// Canonical checksum alphabet.
///
/// Ambiguous glyphs are deliberately excluded where practical.
const CHECKSUM_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Structured representation of a parsed AOXC actor identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActorIdParts {
    /// Protocol prefix. Expected to be `AOXC`.
    pub prefix: String,

    /// Canonical 3-character role code.
    pub role: String,

    /// Canonical 2-character zone code.
    pub zone: String,

    /// Canonical uppercase hexadecimal serial.
    pub serial: String,

    /// Canonical uppercase checksum.
    pub checksum: String,
}

/// Canonical error surface for actor-id derivation and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ActorIdError {
    /// The supplied public key is empty.
    EmptyPublicKey,

    /// The actor-id string is empty or whitespace only.
    EmptyActorId,

    /// The actor-id string has an invalid number of structural components.
    InvalidFormat,

    /// The actor-id prefix is invalid.
    InvalidPrefix,

    /// The normalized role value is invalid.
    InvalidRole,

    /// The normalized zone value is invalid.
    InvalidZone,

    /// The serial component is malformed.
    InvalidSerial,

    /// The checksum component is malformed.
    InvalidChecksum,

    /// The actor-id checksum does not match the canonical derivation.
    ChecksumMismatch,

    /// The actor-id contains unsupported characters.
    InvalidCharacter,

    /// The actor-id does not match the expected derived value for the provided inputs.
    DerivationMismatch,
}

impl ActorIdError {
    /// Returns a stable symbolic error code suitable for logging and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyPublicKey => "ACTOR_ID_EMPTY_PUBLIC_KEY",
            Self::EmptyActorId => "ACTOR_ID_EMPTY",
            Self::InvalidFormat => "ACTOR_ID_INVALID_FORMAT",
            Self::InvalidPrefix => "ACTOR_ID_INVALID_PREFIX",
            Self::InvalidRole => "ACTOR_ID_INVALID_ROLE",
            Self::InvalidZone => "ACTOR_ID_INVALID_ZONE",
            Self::InvalidSerial => "ACTOR_ID_INVALID_SERIAL",
            Self::InvalidChecksum => "ACTOR_ID_INVALID_CHECKSUM",
            Self::ChecksumMismatch => "ACTOR_ID_CHECKSUM_MISMATCH",
            Self::InvalidCharacter => "ACTOR_ID_INVALID_CHARACTER",
            Self::DerivationMismatch => "ACTOR_ID_DERIVATION_MISMATCH",
        }
    }
}

impl fmt::Display for ActorIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPublicKey => {
                write!(
                    f,
                    "actor-id generation failed: public key must not be empty"
                )
            }
            Self::EmptyActorId => {
                write!(f, "actor-id validation failed: actor id must not be empty")
            }
            Self::InvalidFormat => {
                write!(f, "actor-id validation failed: actor id format is invalid")
            }
            Self::InvalidPrefix => {
                write!(f, "actor-id validation failed: prefix is invalid")
            }
            Self::InvalidRole => {
                write!(f, "actor-id validation failed: role component is invalid")
            }
            Self::InvalidZone => {
                write!(f, "actor-id validation failed: zone component is invalid")
            }
            Self::InvalidSerial => {
                write!(f, "actor-id validation failed: serial component is invalid")
            }
            Self::InvalidChecksum => {
                write!(
                    f,
                    "actor-id validation failed: checksum component is invalid"
                )
            }
            Self::ChecksumMismatch => {
                write!(f, "actor-id validation failed: checksum mismatch detected")
            }
            Self::InvalidCharacter => {
                write!(
                    f,
                    "actor-id validation failed: unsupported character detected"
                )
            }
            Self::DerivationMismatch => {
                write!(
                    f,
                    "actor-id validation failed: derived actor id does not match expected value"
                )
            }
        }
    }
}

impl std::error::Error for ActorIdError {}

/// Generates a canonical AOXC actor identifier.
///
/// Canonical derivation inputs:
/// - actor-id domain namespace
/// - normalized role
/// - normalized zone
/// - raw public key bytes
///
/// The returned identifier includes:
/// - canonical prefix
/// - normalized role
/// - normalized zone
/// - deterministic uppercase hexadecimal serial
/// - compact checksum
pub fn generate_actor_id(pubkey: &[u8], role: &str, zone: &str) -> Result<String, ActorIdError> {
    if pubkey.is_empty() {
        return Err(ActorIdError::EmptyPublicKey);
    }

    let role = normalize_role(role)?;
    let zone = normalize_zone(zone)?;
    let serial = derive_serial(pubkey, &role, &zone);
    let checksum = derive_checksum(&role, &zone, &serial);

    Ok(format!(
        "{}-{}-{}-{}-{}",
        AOXC_PREFIX, role, zone, serial, checksum
    ))
}

/// Generates and immediately validates a canonical actor identifier.
///
/// This helper is useful in production call paths where derivation failures
/// and post-derivation format drift must be rejected as a hard invariant.
pub fn generate_and_validate_actor_id(
    pubkey: &[u8],
    role: &str,
    zone: &str,
) -> Result<String, ActorIdError> {
    let actor_id = generate_actor_id(pubkey, role, zone)?;
    validate_actor_id(&actor_id)?;
    Ok(actor_id)
}

/// Validates a canonical AOXC actor identifier.
///
/// Validation includes:
/// - basic structure
/// - prefix
/// - role and zone width and character policy
/// - serial format
/// - checksum format
/// - checksum correctness
pub fn validate_actor_id(actor_id: &str) -> Result<(), ActorIdError> {
    let parts = parse_actor_id(actor_id)?;

    if parts.prefix != AOXC_PREFIX {
        return Err(ActorIdError::InvalidPrefix);
    }

    validate_role_component(&parts.role)?;
    validate_zone_component(&parts.zone)?;
    validate_serial_component(&parts.serial)?;
    validate_checksum_component(&parts.checksum)?;

    let expected_checksum = derive_checksum(&parts.role, &parts.zone, &parts.serial);
    if parts.checksum != expected_checksum {
        return Err(ActorIdError::ChecksumMismatch);
    }

    Ok(())
}

/// Parses an AOXC actor identifier into structured parts.
///
/// This function performs structural parsing and basic character screening.
/// Full semantic validation should be performed with [`validate_actor_id`].
pub fn parse_actor_id(actor_id: &str) -> Result<ActorIdParts, ActorIdError> {
    let candidate = actor_id.trim();

    if candidate.is_empty() {
        return Err(ActorIdError::EmptyActorId);
    }

    if !candidate
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return Err(ActorIdError::InvalidCharacter);
    }

    let parts: Vec<&str> = candidate.split('-').collect();
    if parts.len() != PART_COUNT {
        return Err(ActorIdError::InvalidFormat);
    }

    let parsed = ActorIdParts {
        prefix: parts[0].to_ascii_uppercase(),
        role: parts[1].to_ascii_uppercase(),
        zone: parts[2].to_ascii_uppercase(),
        serial: parts[3].to_ascii_uppercase(),
        checksum: parts[4].to_ascii_uppercase(),
    };

    Ok(parsed)
}

/// Verifies that an actor identifier matches the expected canonical derivation
/// for the provided public key, role, and zone inputs.
pub fn verify_actor_id_binding(
    actor_id: &str,
    pubkey: &[u8],
    role: &str,
    zone: &str,
) -> Result<(), ActorIdError> {
    validate_actor_id(actor_id)?;

    let expected = generate_actor_id(pubkey, role, zone)?;
    if actor_id.trim().to_ascii_uppercase() != expected {
        return Err(ActorIdError::DerivationMismatch);
    }

    Ok(())
}

/// Returns a canonical normalized role code.
///
/// Policy:
/// - uppercase ASCII only
/// - unsupported characters removed
/// - strict fixed width
/// - empty normalized output rejected
fn normalize_role(role: &str) -> Result<String, ActorIdError> {
    normalize_component(role, ROLE_LEN).ok_or(ActorIdError::InvalidRole)
}

/// Returns a canonical normalized zone code.
///
/// Policy:
/// - uppercase ASCII only
/// - unsupported characters removed
/// - strict fixed width
/// - empty normalized output rejected
fn normalize_zone(zone: &str) -> Result<String, ActorIdError> {
    normalize_component(zone, ZONE_LEN).ok_or(ActorIdError::InvalidZone)
}

/// Normalizes a textual identifier into a fixed-width uppercase alphanumeric token.
///
/// Rules:
/// - converts to uppercase
/// - removes non-ASCII-alphanumeric characters
/// - truncates if longer than `len`
/// - pads with `X` if shorter than `len`
///
/// Returns `None` if no valid alphanumeric characters remain after normalization.
fn normalize_component(value: &str, len: usize) -> Option<String> {
    let mut out = value
        .to_ascii_uppercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .take(len)
        .collect::<String>();

    if out.is_empty() {
        return None;
    }

    while out.len() < len {
        out.push('X');
    }

    Some(out)
}

/// Derives the visible actor serial from the public key and canonical context.
///
/// The serial is derived from:
/// - actor-id namespace
/// - normalized role
/// - normalized zone
/// - public key bytes
///
/// The resulting digest is truncated to `SERIAL_BYTES` and rendered as
/// uppercase hexadecimal.
fn derive_serial(pubkey: &[u8], role: &str, zone: &str) -> String {
    let mut hasher = Sha3_256::new();

    hasher.update(ACTOR_ID_DOMAIN);
    hasher.update([0x00]);
    hasher.update(role.as_bytes());
    hasher.update([0x00]);
    hasher.update(zone.as_bytes());
    hasher.update([0x00]);
    hasher.update(pubkey);

    let hash = hasher.finalize();
    hex::encode_upper(&hash[..SERIAL_BYTES])
}

/// Derives the compact checksum for a canonical actor identifier.
///
/// The checksum is derived from:
/// - checksum namespace
/// - canonical prefix
/// - canonical role
/// - canonical zone
/// - canonical serial
///
/// The resulting checksum is encoded using a restricted alphabet for improved
/// operator readability.
fn derive_checksum(role: &str, zone: &str, serial: &str) -> String {
    let mut hasher = Sha3_256::new();

    hasher.update(ACTOR_ID_CHECKSUM_DOMAIN);
    hasher.update([0x00]);
    hasher.update(AOXC_PREFIX.as_bytes());
    hasher.update([0x00]);
    hasher.update(role.as_bytes());
    hasher.update([0x00]);
    hasher.update(zone.as_bytes());
    hasher.update([0x00]);
    hasher.update(serial.as_bytes());

    let digest = hasher.finalize();
    encode_checksum(&digest[..CHECKSUM_LEN])
}

/// Encodes raw checksum bytes into a compact operator-friendly alphabet.
fn encode_checksum(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());

    for byte in bytes {
        let idx = (*byte as usize) % CHECKSUM_ALPHABET.len();
        out.push(CHECKSUM_ALPHABET[idx] as char);
    }

    out
}

/// Validates the canonical role component.
fn validate_role_component(role: &str) -> Result<(), ActorIdError> {
    if role.len() != ROLE_LEN || !role.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(ActorIdError::InvalidRole);
    }

    Ok(())
}

/// Validates the canonical zone component.
fn validate_zone_component(zone: &str) -> Result<(), ActorIdError> {
    if zone.len() != ZONE_LEN || !zone.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        return Err(ActorIdError::InvalidZone);
    }

    Ok(())
}

/// Validates the canonical serial component.
fn validate_serial_component(serial: &str) -> Result<(), ActorIdError> {
    if serial.len() != SERIAL_HEX_LEN {
        return Err(ActorIdError::InvalidSerial);
    }

    if !serial.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ActorIdError::InvalidSerial);
    }

    Ok(())
}

/// Validates the canonical checksum component.
fn validate_checksum_component(checksum: &str) -> Result<(), ActorIdError> {
    if checksum.len() != CHECKSUM_LEN {
        return Err(ActorIdError::InvalidChecksum);
    }

    if !checksum
        .chars()
        .all(|ch| CHECKSUM_ALPHABET.contains(&(ch as u8)))
    {
        return Err(ActorIdError::InvalidChecksum);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pubkey() -> Vec<u8> {
        vec![0x11; 32]
    }

    #[test]
    fn actor_id_generation_is_deterministic() {
        let pk = sample_pubkey();

        let a = generate_actor_id(&pk, "val", "eu").expect("actor id generation must succeed");
        let b = generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        assert_eq!(a, b);
    }

    #[test]
    fn actor_id_contains_expected_structure() {
        let pk = sample_pubkey();
        let actor_id = generate_actor_id(&pk, "validator", "europe")
            .expect("actor id generation must succeed");

        let parts: Vec<&str> = actor_id.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0], AOXC_PREFIX);
        assert_eq!(parts[1].len(), ROLE_LEN);
        assert_eq!(parts[2].len(), ZONE_LEN);
        assert_eq!(parts[3].len(), SERIAL_HEX_LEN);
        assert_eq!(parts[4].len(), CHECKSUM_LEN);
    }

    #[test]
    fn actor_id_validation_accepts_valid_identifier() {
        let pk = sample_pubkey();
        let actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        assert_eq!(validate_actor_id(&actor_id), Ok(()));
    }

    #[test]
    fn actor_id_validation_rejects_checksum_mismatch() {
        let pk = sample_pubkey();
        let mut actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        actor_id.pop();
        actor_id.push('Z');

        assert_eq!(
            validate_actor_id(&actor_id),
            Err(ActorIdError::ChecksumMismatch)
        );
    }

    #[test]
    fn actor_id_binding_verification_accepts_matching_inputs() {
        let pk = sample_pubkey();
        let actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        assert_eq!(verify_actor_id_binding(&actor_id, &pk, "VAL", "EU"), Ok(()));
    }

    #[test]
    fn actor_id_binding_verification_rejects_wrong_zone() {
        let pk = sample_pubkey();
        let actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        assert_eq!(
            verify_actor_id_binding(&actor_id, &pk, "VAL", "NA"),
            Err(ActorIdError::DerivationMismatch)
        );
    }

    #[test]
    fn empty_public_key_is_rejected() {
        assert_eq!(
            generate_actor_id(&[], "VAL", "EU"),
            Err(ActorIdError::EmptyPublicKey)
        );
    }

    #[test]
    fn parse_actor_id_extracts_components() {
        let pk = sample_pubkey();
        let actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        let parsed = parse_actor_id(&actor_id).expect("actor id parsing must succeed");
        assert_eq!(parsed.prefix, "AOXC");
        assert_eq!(parsed.role, "VAL");
        assert_eq!(parsed.zone, "EU");
        assert_eq!(parsed.serial.len(), SERIAL_HEX_LEN);
        assert_eq!(parsed.checksum.len(), CHECKSUM_LEN);
    }

    #[test]
    fn normalization_pads_short_components() {
        let pk = sample_pubkey();
        let actor_id = generate_actor_id(&pk, "v", "e").expect("actor id generation must succeed");

        let parsed = parse_actor_id(&actor_id).expect("actor id parsing must succeed");
        assert_eq!(parsed.role, "VXX");
        assert_eq!(parsed.zone, "EX");
    }

    #[test]
    fn generation_rejects_fully_invalid_role() {
        let pk = sample_pubkey();
        assert_eq!(
            generate_actor_id(&pk, "!!!", "EU"),
            Err(ActorIdError::InvalidRole)
        );
    }

    #[test]
    fn validation_rejects_invalid_prefix() {
        let actor_id = "NOPE-VAL-EU-3F7A9C21D4E8B7AA-K9";
        assert_eq!(
            validate_actor_id(actor_id),
            Err(ActorIdError::InvalidPrefix)
        );
    }
}
