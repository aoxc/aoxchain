// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC Actor ID Derivation and Validation Module.
//!
//! Canonical format:
//!
//! AOXC-RRR-ZZ-SSSSSSSSSSSSSSSSSSSSSSSS-CC
//!
//! Example:
//!
//! AOXC-VAL-EU-3F7A9C21D4E8B7AA6C019A52-K9
//!
//! Where:
//! - AOXC : protocol prefix
//! - RRR  : canonical 3-character role code
//! - ZZ   : canonical 2-character zone code
//! - S... : 24 uppercase hexadecimal characters derived deterministically
//! - CC   : 2-character checksum for operator-facing typo detection
//!
//! Security and design objectives:
//! - deterministic derivation from canonical input,
//! - strict canonical formatting,
//! - stable serialization shape,
//! - operator readability,
//! - explicit domain separation,
//! - post-quantum-friendly hash primitives,
//! - reduced ambiguity compared to permissive normalization.
//!
//! Important note:
//! This module provides canonical identifier derivation and validation only.
//! It does not by itself prove cryptographic ownership of the referenced key.

use sha3::{
    Shake256,
    digest::{ExtendableOutput, Update, XofReader},
};
use std::fmt;

/// Canonical AOXC actor identifier prefix.
pub const AOXC_PREFIX: &str = "AOXC";

/// Canonical actor-id derivation domain separator.
const ACTOR_ID_DOMAIN: &[u8] = b"AOXC/IDENTITY/ACTOR_ID/V1";

/// Canonical actor-id checksum domain separator.
const ACTOR_ID_CHECKSUM_DOMAIN: &[u8] = b"AOXC/IDENTITY/ACTOR_ID/CHECKSUM/V1";

/// Canonical actor-id protocol context.
///
/// This value is intentionally protocol-scoped rather than environment-scoped.
/// It avoids accidental hard-binding to a single deployment profile such as
/// mainnet while preserving explicit AOXC namespace separation.
const ACTOR_ID_PROTOCOL_CONTEXT: &[u8] = b"AOXC/IDENTITY/PROTOCOL/V1";

/// Canonical role width.
pub const ROLE_LEN: usize = 3;

/// Canonical zone width.
pub const ZONE_LEN: usize = 2;

/// Number of digest bytes used for the visible serial portion.
///
/// 12 bytes => 24 uppercase hexadecimal characters.
pub const SERIAL_BYTES: usize = 12;

/// Canonical visible serial length in hexadecimal characters.
pub const SERIAL_HEX_LEN: usize = SERIAL_BYTES * 2;

/// Number of checksum symbols appended to the actor identifier.
pub const CHECKSUM_LEN: usize = 2;

/// Total number of dash-separated components in the canonical actor id.
const PART_COUNT: usize = 5;

/// Minimum accepted public-key byte length.
///
/// This lower bound intentionally allows multiple cryptographic families
/// while rejecting clearly malformed or degenerate inputs.
pub const MIN_PUBLIC_KEY_LEN: usize = 16;

/// Maximum accepted public-key byte length.
///
/// This upper bound is intentionally generous to remain compatible with
/// larger post-quantum public key surfaces.
pub const MAX_PUBLIC_KEY_LEN: usize = 4096;

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

    /// The supplied public key length is outside the accepted policy range.
    InvalidPublicKeyLength,

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
            Self::InvalidPublicKeyLength => "ACTOR_ID_INVALID_PUBLIC_KEY_LENGTH",
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
            Self::InvalidPublicKeyLength => {
                write!(
                    f,
                    "actor-id generation failed: public key length is outside the accepted policy range"
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
/// - actor-id namespace,
/// - protocol context,
/// - canonical role,
/// - canonical zone,
/// - public-key length,
/// - raw public key bytes.
///
/// The returned identifier is guaranteed to pass strict canonical validation.
pub fn generate_actor_id(pubkey: &[u8], role: &str, zone: &str) -> Result<String, ActorIdError> {
    validate_public_key(pubkey)?;

    let canonical_role = canonicalize_role(role)?;
    let canonical_zone = canonicalize_zone(zone)?;
    let serial = derive_serial(pubkey, &canonical_role, &canonical_zone);
    let checksum = derive_checksum(&canonical_role, &canonical_zone, &serial);

    let actor_id = format!(
        "{}-{}-{}-{}-{}",
        AOXC_PREFIX, canonical_role, canonical_zone, serial, checksum
    );

    validate_actor_id(&actor_id)?;
    Ok(actor_id)
}

/// Generates and immediately validates a canonical actor identifier.
///
/// This helper preserves a hard invariant that the generation path and the
/// strict validation path remain mutually consistent.
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
/// - strict structure,
/// - canonical uppercase formatting,
/// - prefix,
/// - role and zone width and character policy,
/// - serial format,
/// - checksum format,
/// - checksum correctness.
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
/// This function performs strict structural parsing. It does not normalize case
/// or silently repair malformed input.
pub fn parse_actor_id(actor_id: &str) -> Result<ActorIdParts, ActorIdError> {
    if actor_id.trim().is_empty() {
        return Err(ActorIdError::EmptyActorId);
    }

    if actor_id != actor_id.trim() {
        return Err(ActorIdError::InvalidFormat);
    }

    if !actor_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return Err(ActorIdError::InvalidCharacter);
    }

    let parts: Vec<&str> = actor_id.split('-').collect();
    if parts.len() != PART_COUNT {
        return Err(ActorIdError::InvalidFormat);
    }

    Ok(ActorIdParts {
        prefix: parts[0].to_string(),
        role: parts[1].to_string(),
        zone: parts[2].to_string(),
        serial: parts[3].to_string(),
        checksum: parts[4].to_string(),
    })
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
    if actor_id != expected {
        return Err(ActorIdError::DerivationMismatch);
    }

    Ok(())
}

/// Validates the public key according to the accepted AOXC actor-id policy.
fn validate_public_key(pubkey: &[u8]) -> Result<(), ActorIdError> {
    if pubkey.is_empty() {
        return Err(ActorIdError::EmptyPublicKey);
    }

    if !(MIN_PUBLIC_KEY_LEN..=MAX_PUBLIC_KEY_LEN).contains(&pubkey.len()) {
        return Err(ActorIdError::InvalidPublicKeyLength);
    }

    Ok(())
}

/// Canonicalizes an operator-supplied role input into a strict 3-character role code.
///
/// Policy:
/// - known aliases are mapped explicitly,
/// - already-canonical 3-character alphanumeric values are accepted,
/// - permissive stripping, truncation, and padding are intentionally forbidden.
fn canonicalize_role(role: &str) -> Result<String, ActorIdError> {
    let normalized = role.trim().to_ascii_lowercase();

    let canonical = match normalized.as_str() {
        "val" | "validator" => "VAL",
        "obs" | "observer" => "OBS",
        "opr" | "operator" => "OPR",
        "seq" | "sequencer" => "SEQ",
        "rly" | "relayer" => "RLY",
        "bld" | "builder" => "BLD",
        "gov" | "governance" => "GOV",
        _ => {
            if role.len() == ROLE_LEN && role.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                return Ok(role.to_ascii_uppercase());
            }
            return Err(ActorIdError::InvalidRole);
        }
    };

    Ok(canonical.to_string())
}

/// Canonicalizes an operator-supplied zone input into a strict 2-character zone code.
///
/// Policy:
/// - known aliases are mapped explicitly,
/// - already-canonical 2-character alphanumeric values are accepted,
/// - permissive stripping, truncation, and padding are intentionally forbidden.
fn canonicalize_zone(zone: &str) -> Result<String, ActorIdError> {
    let normalized = zone.trim().to_ascii_lowercase();

    let canonical = match normalized.as_str() {
        "eu" | "europe" => "EU",
        "na" | "northamerica" | "north-america" => "NA",
        "ap" | "apac" | "asia-pacific" | "asiapacific" => "AP",
        "sa" | "southamerica" | "south-america" => "SA",
        "af" | "africa" => "AF",
        "me" | "middleeast" | "middle-east" => "ME",
        "oc" | "oceania" => "OC",
        "gl" | "global" => "GL",
        _ => {
            if zone.len() == ZONE_LEN && zone.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                return Ok(zone.to_ascii_uppercase());
            }
            return Err(ActorIdError::InvalidZone);
        }
    };

    Ok(canonical.to_string())
}

/// Derives the visible actor serial from canonical input.
///
/// The serial is derived from:
/// - actor-id namespace,
/// - protocol context,
/// - canonical role,
/// - canonical zone,
/// - big-endian public-key length,
/// - raw public key bytes.
///
/// The resulting XOF output is truncated to `SERIAL_BYTES` and rendered as
/// uppercase hexadecimal.
fn derive_serial(pubkey: &[u8], role: &str, zone: &str) -> String {
    let mut shake = Shake256::default();
    shake.update(ACTOR_ID_DOMAIN);
    shake.update(&[0x00]);
    shake.update(ACTOR_ID_PROTOCOL_CONTEXT);
    shake.update(&[0x00]);
    shake.update(role.as_bytes());
    shake.update(&[0x00]);
    shake.update(zone.as_bytes());
    shake.update(&[0x00]);
    shake.update(&(pubkey.len() as u32).to_be_bytes());
    shake.update(&[0x00]);
    shake.update(pubkey);

    let mut reader = shake.finalize_xof();
    let mut serial_bytes = [0u8; SERIAL_BYTES];
    reader.read(&mut serial_bytes);

    hex::encode_upper(serial_bytes)
}

/// Derives the compact checksum for a canonical actor identifier.
///
/// The checksum is derived from:
/// - checksum namespace,
/// - canonical prefix,
/// - canonical role,
/// - canonical zone,
/// - canonical serial.
fn derive_checksum(role: &str, zone: &str, serial: &str) -> String {
    let mut shake = Shake256::default();
    shake.update(ACTOR_ID_CHECKSUM_DOMAIN);
    shake.update(&[0x00]);
    shake.update(ACTOR_ID_PROTOCOL_CONTEXT);
    shake.update(&[0x00]);
    shake.update(AOXC_PREFIX.as_bytes());
    shake.update(&[0x00]);
    shake.update(role.as_bytes());
    shake.update(&[0x00]);
    shake.update(zone.as_bytes());
    shake.update(&[0x00]);
    shake.update(serial.as_bytes());

    let mut reader = shake.finalize_xof();
    let mut checksum_bytes = [0u8; CHECKSUM_LEN];
    reader.read(&mut checksum_bytes);

    encode_checksum(&checksum_bytes)
}

/// Encodes raw checksum bytes into the restricted operator-friendly alphabet.
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
    if role.len() != ROLE_LEN
        || !role
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return Err(ActorIdError::InvalidRole);
    }

    Ok(())
}

/// Validates the canonical zone component.
fn validate_zone_component(zone: &str) -> Result<(), ActorIdError> {
    if zone.len() != ZONE_LEN
        || !zone
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
    {
        return Err(ActorIdError::InvalidZone);
    }

    Ok(())
}

/// Validates the canonical serial component.
fn validate_serial_component(serial: &str) -> Result<(), ActorIdError> {
    if serial.len() != SERIAL_HEX_LEN {
        return Err(ActorIdError::InvalidSerial);
    }

    if !serial
        .chars()
        .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_lowercase())
    {
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
