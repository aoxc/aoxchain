// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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

use sha3::{
    Digest, Sha3_256, Shake256,
    digest::{ExtendableOutput, XofReader},
};
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

/// AOXC chain context byte namespace.
///
/// This value hard-binds derivation to AOXC chain semantics and prevents
/// accidental reuse inside sibling protocols.
const AOXC_CHAIN_CONTEXT: &[u8] = b"AOXC/CHAIN/MAINNET";

/// Canonical role width.
pub const ROLE_LEN: usize = 3;

/// Canonical zone width.
pub const ZONE_LEN: usize = 2;

/// Number of digest bytes used for the visible serial portion.
///
/// 12 bytes => 24 uppercase hexadecimal characters.
/// This expands the visible actor-id namespace while preserving readability.
pub const SERIAL_BYTES: usize = 12;

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
    // Quantum-aware hybrid sponge:
    // 1) Fixed-output SHA3-256 for compatibility and deterministic stability.
    // 2) SHAKE256 XOF stream for expanded entropy extraction and future-proofing.
    // 3) XOR fusion to bind both digests into a single serial material.
    let mut sha3 = Sha3_256::new();
    sha3.update(ACTOR_ID_DOMAIN);
    sha3.update([0x00]);
    sha3.update(AOXC_CHAIN_CONTEXT);
    sha3.update([0x00]);
    sha3.update(role.as_bytes());
    sha3.update([0x00]);
    sha3.update(zone.as_bytes());
    sha3.update([0x00]);
    sha3.update([(pubkey.len() & 0xFF) as u8]);
    sha3.update([0x00]);
    sha3.update(pubkey);
    let sha3_digest = sha3.finalize();

    let mut shake = Shake256::default();
    sha3::digest::Update::update(&mut shake, ACTOR_ID_DOMAIN);
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, AOXC_CHAIN_CONTEXT);
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, role.as_bytes());
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, zone.as_bytes());
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, &[(pubkey.len() & 0xFF) as u8]);
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, pubkey);

    let mut xof = shake.finalize_xof();
    let mut shake_out = vec![0_u8; SERIAL_BYTES];
    xof.read(&mut shake_out);

    let fused = (0..SERIAL_BYTES)
        .map(|idx| shake_out[idx] ^ sha3_digest[idx % sha3_digest.len()])
        .collect::<Vec<u8>>();

    hex::encode_upper(fused)
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
    let mut shake = Shake256::default();
    sha3::digest::Update::update(&mut shake, ACTOR_ID_CHECKSUM_DOMAIN);
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, AOXC_CHAIN_CONTEXT);
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, AOXC_PREFIX.as_bytes());
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, role.as_bytes());
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, zone.as_bytes());
    sha3::digest::Update::update(&mut shake, &[0x00]);
    sha3::digest::Update::update(&mut shake, serial.as_bytes());

    let mut xof = shake.finalize_xof();
    let mut checksum_bytes = [0_u8; CHECKSUM_LEN];
    xof.read(&mut checksum_bytes);
    encode_checksum(&checksum_bytes)
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
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

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

    #[test]
    fn generate_and_validate_actor_id_matches_validation_path() {
        let pk = sample_pubkey();
        let actor_id =
            generate_and_validate_actor_id(&pk, "validator", "europe").expect("must succeed");

        assert_eq!(validate_actor_id(&actor_id), Ok(()));
    }

    #[test]
    fn parse_rejects_whitespace_only_identifier() {
        assert_eq!(parse_actor_id("   "), Err(ActorIdError::EmptyActorId));
    }

    #[test]
    fn parse_rejects_structural_component_count_mismatch() {
        assert_eq!(
            parse_actor_id("AOXC-VAL-EU-ABCD"),
            Err(ActorIdError::InvalidFormat)
        );
    }

    #[test]
    fn validation_rejects_invalid_serial_lengths_and_characters() {
        let pk = sample_pubkey();
        let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let mut parts: Vec<&str> = valid.split('-').collect();

        parts[3] = "ABC";
        let bad_len = parts.join("-");
        assert_eq!(
            validate_actor_id(&bad_len),
            Err(ActorIdError::InvalidSerial)
        );

        parts[3] = "ZZZZZZZZZZZZZZZZZZZZZZZZ";
        let bad_chars = parts.join("-");
        assert_eq!(
            validate_actor_id(&bad_chars),
            Err(ActorIdError::InvalidSerial)
        );
    }

    #[test]
    fn validation_rejects_invalid_checksum_symbols() {
        let pk = sample_pubkey();
        let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let mut parts: Vec<&str> = valid.split('-').collect();
        parts[4] = "IO";
        let actor_id = parts.join("-");
        assert_eq!(
            validate_actor_id(&actor_id),
            Err(ActorIdError::InvalidChecksum)
        );
    }

    #[test]
    fn binding_verification_rejects_role_mismatch() {
        let pk = sample_pubkey();
        let actor_id =
            generate_actor_id(&pk, "VAL", "EU").expect("actor id generation must succeed");

        assert_eq!(
            verify_actor_id_binding(&actor_id, &pk, "OBS", "EU"),
            Err(ActorIdError::DerivationMismatch)
        );
    }

    #[test]
    fn deterministic_randomized_actor_id_regression_stress() {
        let mut rng = StdRng::seed_from_u64(0xA0C1_D1D5_u64);

        for _ in 0..1_000 {
            let mut pubkey = [0u8; 32];
            rng.fill_bytes(&mut pubkey);
            if pubkey.iter().all(|b| *b == 0) {
                pubkey[0] = 1;
            }

            let role = if (rng.next_u32() & 1) == 0 {
                "validator"
            } else {
                "observer"
            };
            let zone = if (rng.next_u32() & 1) == 0 {
                "eu"
            } else {
                "na"
            };

            let a = generate_actor_id(&pubkey, role, zone).expect("must succeed");
            let b = generate_actor_id(&pubkey, role, zone).expect("must succeed");

            assert_eq!(a, b);
            assert_eq!(validate_actor_id(&a), Ok(()));
            assert_eq!(verify_actor_id_binding(&a, &pubkey, role, zone), Ok(()));
        }
    }

    #[test]
    fn validate_rejects_invalid_role_and_zone_lengths() {
        let pk = sample_pubkey();
        let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let mut parts: Vec<&str> = valid.split('-').collect();

        parts[1] = "VA";
        let short_role = parts.join("-");
        assert_eq!(
            validate_actor_id(&short_role),
            Err(ActorIdError::InvalidRole)
        );

        parts[1] = "VAL";
        parts[2] = "E";
        let short_zone = parts.join("-");
        assert_eq!(
            validate_actor_id(&short_zone),
            Err(ActorIdError::InvalidZone)
        );
    }

    #[test]
    fn validate_rejects_non_alphanumeric_role_zone_symbols() {
        let pk = sample_pubkey();
        let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let mut parts: Vec<&str> = valid.split('-').collect();

        parts[1] = "V*L";
        let bad_role = parts.join("-");
        assert_eq!(
            validate_actor_id(&bad_role),
            Err(ActorIdError::InvalidCharacter)
        );

        parts[1] = "VAL";
        parts[2] = "E*";
        let bad_zone = parts.join("-");
        assert_eq!(
            validate_actor_id(&bad_zone),
            Err(ActorIdError::InvalidCharacter)
        );
    }

    #[test]
    fn parse_and_validate_accept_mixed_case_inputs() {
        let pk = sample_pubkey();
        let canonical = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let mixed = canonical.to_ascii_lowercase();

        let parsed = parse_actor_id(&mixed).expect("parse should normalize");
        assert_eq!(parsed.prefix, AOXC_PREFIX);
        assert_eq!(validate_actor_id(&mixed), Ok(()));
    }

    #[test]
    fn validate_detects_checksum_mismatch_for_every_checksum_mutation() {
        let pk = sample_pubkey();
        let valid = generate_actor_id(&pk, "VAL", "EU").expect("must succeed");
        let parts: Vec<&str> = valid.split('-').collect();
        let checksum = parts[4];
        let alphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789".chars();

        for i in 0..checksum.len() {
            for ch in alphabet.clone() {
                if checksum.chars().nth(i).unwrap() == ch {
                    continue;
                }

                let mut mutated_checksum: Vec<char> = checksum.chars().collect();
                mutated_checksum[i] = ch;
                let candidate = format!(
                    "{}-{}-{}-{}-{}",
                    parts[0],
                    parts[1],
                    parts[2],
                    parts[3],
                    mutated_checksum.iter().collect::<String>()
                );
                assert_eq!(
                    validate_actor_id(&candidate),
                    Err(ActorIdError::ChecksumMismatch)
                );
            }
        }
    }

    #[test]
    fn parse_rejects_non_ascii_or_space_symbols() {
        assert_eq!(
            parse_actor_id("AOXC-VAL-EU-ABC DEF-K9"),
            Err(ActorIdError::InvalidCharacter)
        );
        assert_eq!(
            parse_actor_id("AOXC-VAL-EU-ABCDEF🙂-K9"),
            Err(ActorIdError::InvalidCharacter)
        );
    }
}
