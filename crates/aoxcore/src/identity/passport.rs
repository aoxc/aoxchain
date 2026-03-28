// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt;

/// Current passport format version.
pub const PASSPORT_VERSION: u8 = 1;

/// Maximum accepted actor identifier length.
const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum accepted role length.
const MAX_ROLE_LEN: usize = 32;

/// Maximum accepted zone length.
const MAX_ZONE_LEN: usize = 32;

/// Maximum accepted embedded certificate length.
///
/// The certificate field currently remains string-based for compatibility with
/// the existing persistence surface.
const MAX_CERTIFICATE_LEN: usize = 65_536;

/// Domain separator for AOXC passport fingerprint derivation.
const PASSPORT_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/PASSPORT/FINGERPRINT/V1";

/// Domain separator for certificate-string fingerprint derivation.
const PASSPORT_CERTIFICATE_FINGERPRINT_DOMAIN: &[u8] =
    b"AOXC/IDENTITY/PASSPORT/CERTIFICATE_FINGERPRINT/V1";

/// Current lifecycle classification for a passport at a specific timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassportValidityState {
    NotYetValid,
    Valid,
    Expired,
}

/// Canonical error surface for AOXC passport validation and serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PassportError {
    InvalidVersion,
    EmptyActorId,
    InvalidActorId,
    EmptyRole,
    InvalidRole,
    EmptyZone,
    InvalidZone,
    EmptyCertificate,
    InvalidCertificate,
    InvalidIssuedAt,
    InvalidExpiresAt,
    InvalidValidityWindow,
    SerializationFailed(String),
    ParseFailed(String),
}

impl PassportError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidVersion => "PASSPORT_INVALID_VERSION",
            Self::EmptyActorId => "PASSPORT_EMPTY_ACTOR_ID",
            Self::InvalidActorId => "PASSPORT_INVALID_ACTOR_ID",
            Self::EmptyRole => "PASSPORT_EMPTY_ROLE",
            Self::InvalidRole => "PASSPORT_INVALID_ROLE",
            Self::EmptyZone => "PASSPORT_EMPTY_ZONE",
            Self::InvalidZone => "PASSPORT_INVALID_ZONE",
            Self::EmptyCertificate => "PASSPORT_EMPTY_CERTIFICATE",
            Self::InvalidCertificate => "PASSPORT_INVALID_CERTIFICATE",
            Self::InvalidIssuedAt => "PASSPORT_INVALID_ISSUED_AT",
            Self::InvalidExpiresAt => "PASSPORT_INVALID_EXPIRES_AT",
            Self::InvalidValidityWindow => "PASSPORT_INVALID_VALIDITY_WINDOW",
            Self::SerializationFailed(_) => "PASSPORT_SERIALIZATION_FAILED",
            Self::ParseFailed(_) => "PASSPORT_PARSE_FAILED",
        }
    }
}

impl fmt::Display for PassportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion => write!(f, "passport validation failed: unsupported version"),
            Self::EmptyActorId => {
                write!(f, "passport validation failed: actor_id must not be empty")
            }
            Self::InvalidActorId => {
                write!(f, "passport validation failed: actor_id format is invalid")
            }
            Self::EmptyRole => write!(f, "passport validation failed: role must not be empty"),
            Self::InvalidRole => write!(f, "passport validation failed: role format is invalid"),
            Self::EmptyZone => write!(f, "passport validation failed: zone must not be empty"),
            Self::InvalidZone => write!(f, "passport validation failed: zone format is invalid"),
            Self::EmptyCertificate => write!(
                f,
                "passport validation failed: certificate must not be empty"
            ),
            Self::InvalidCertificate => write!(
                f,
                "passport validation failed: certificate format is invalid"
            ),
            Self::InvalidIssuedAt => write!(f, "passport validation failed: issued_at is invalid"),
            Self::InvalidExpiresAt => {
                write!(f, "passport validation failed: expires_at is invalid")
            }
            Self::InvalidValidityWindow => {
                write!(
                    f,
                    "passport validation failed: expires_at must be greater than issued_at"
                )
            }
            Self::SerializationFailed(error) => {
                write!(f, "passport serialization failed: {}", error)
            }
            Self::ParseFailed(error) => {
                write!(f, "passport parsing failed: {}", error)
            }
        }
    }
}

impl std::error::Error for PassportError {}

/// Represents a node identity passport.
///
/// A passport bundles actor metadata together with its certificate and minimal
/// runtime identity information used during handshake.
///
/// Compatibility notes:
/// - the field layout is preserved,
/// - the certificate remains string-based,
/// - validation helpers are added without changing the persisted shape.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Passport {
    pub version: u8,
    pub actor_id: String,
    pub role: String,
    pub zone: String,
    pub certificate: String,
    pub issued_at: u64,
    pub expires_at: u64,
}

impl Passport {
    /// Creates a new passport.
    ///
    /// Construction remains infallible for compatibility with existing call
    /// sites. Validation should be enforced through `validate()` or `from_json()`.
    #[must_use]
    pub fn new(
        actor_id: String,
        role: String,
        zone: String,
        certificate: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Self {
        Self {
            version: PASSPORT_VERSION,
            actor_id,
            role,
            zone,
            certificate,
            issued_at,
            expires_at,
        }
    }

    /// Validates the full passport structure.
    pub fn validate(&self) -> Result<(), PassportError> {
        if self.version != PASSPORT_VERSION {
            return Err(PassportError::InvalidVersion);
        }

        validate_actor_id(&self.actor_id)?;
        validate_role(&self.role)?;
        validate_zone(&self.zone)?;
        validate_certificate(&self.certificate)?;
        validate_validity_window(self.issued_at, self.expires_at)?;

        Ok(())
    }

    /// Returns true if the passport has expired at the supplied timestamp.
    ///
    /// Boundary policy:
    /// - a passport is treated as expired once `now >= expires_at`.
    #[must_use]
    pub fn is_expired(&self, now: u64) -> bool {
        now >= self.expires_at
    }

    /// Returns true if the passport is not yet valid at the supplied timestamp.
    #[must_use]
    pub fn is_not_yet_valid_at(&self, now: u64) -> bool {
        now < self.issued_at
    }

    /// Returns true if the passport is valid at the supplied timestamp.
    #[must_use]
    pub fn is_valid_at(&self, now: u64) -> bool {
        now >= self.issued_at && now < self.expires_at
    }

    /// Returns the lifecycle classification of the passport at the supplied timestamp.
    #[must_use]
    pub fn validity_state_at(&self, now: u64) -> PassportValidityState {
        if self.is_not_yet_valid_at(now) {
            PassportValidityState::NotYetValid
        } else if self.is_expired(now) {
            PassportValidityState::Expired
        } else {
            PassportValidityState::Valid
        }
    }

    /// Computes a deterministic fingerprint for the passport.
    ///
    /// This implementation intentionally avoids fallible JSON serialization in
    /// the fingerprint path and instead hashes the canonical field sequence
    /// directly under an explicit AOXC domain separator.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha3_256::new();

        hasher.update(PASSPORT_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);

        hasher.update([self.version]);
        hasher.update([0x00]);

        hasher.update(self.actor_id.as_bytes());
        hasher.update([0x00]);

        hasher.update(self.role.as_bytes());
        hasher.update([0x00]);

        hasher.update(self.zone.as_bytes());
        hasher.update([0x00]);

        hasher.update(self.certificate.as_bytes());
        hasher.update([0x00]);

        hasher.update(self.issued_at.to_be_bytes());
        hasher.update([0x00]);

        hasher.update(self.expires_at.to_be_bytes());

        let digest = hasher.finalize();
        hex::encode_upper(&digest[..8])
    }

    /// Computes a deterministic fingerprint of the embedded certificate string.
    ///
    /// This helper is useful when operators want a short reference to the
    /// certificate payload without hashing the full passport object.
    #[must_use]
    pub fn certificate_fingerprint(&self) -> String {
        let mut hasher = Sha3_256::new();

        hasher.update(PASSPORT_CERTIFICATE_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(self.certificate.as_bytes());

        let digest = hasher.finalize();
        hex::encode_upper(&digest[..8])
    }

    /// Serializes the passport to JSON.
    ///
    /// The passport is validated before serialization so that invalid objects
    /// do not silently leave the process boundary.
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()
            .map_err(|error| format!("PASSPORT_VALIDATE_ERROR: {}", error.code()))?;

        serde_json::to_string(self).map_err(|error| format!("PASSPORT_SERIALIZE_ERROR: {}", error))
    }

    /// Restores a passport from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, String> {
        let passport: Self = serde_json::from_str(data)
            .map_err(|error| format!("PASSPORT_PARSE_ERROR: {}", error))?;

        passport
            .validate()
            .map_err(|error| format!("PASSPORT_VALIDATE_ERROR: {}", error.code()))?;

        Ok(passport)
    }
}

/// Validates the shared issued/expires window.
fn validate_validity_window(issued_at: u64, expires_at: u64) -> Result<(), PassportError> {
    if issued_at == 0 {
        return Err(PassportError::InvalidIssuedAt);
    }

    if expires_at == 0 {
        return Err(PassportError::InvalidExpiresAt);
    }

    if expires_at <= issued_at {
        return Err(PassportError::InvalidValidityWindow);
    }

    Ok(())
}

/// Validates that a string field is non-empty, trimmed, and bounded.
fn validate_canonical_text(
    value: &str,
    max_len: usize,
    empty_error: PassportError,
    invalid_error: PassportError,
    predicate: impl Fn(char) -> bool,
) -> Result<(), PassportError> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(empty_error);
    }

    if value != value.trim() {
        return Err(invalid_error);
    }

    if value.len() > max_len {
        return Err(invalid_error);
    }

    if !value.chars().all(predicate) {
        return Err(invalid_error);
    }

    Ok(())
}

/// Validates the actor_id field.
fn validate_actor_id(value: &str) -> Result<(), PassportError> {
    validate_canonical_text(
        value,
        MAX_ACTOR_ID_LEN,
        PassportError::EmptyActorId,
        PassportError::InvalidActorId,
        |ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.',
    )?;

    if !value.starts_with("AOXC-") {
        return Err(PassportError::InvalidActorId);
    }

    Ok(())
}

/// Validates the role field.
fn validate_role(value: &str) -> Result<(), PassportError> {
    validate_canonical_text(
        value,
        MAX_ROLE_LEN,
        PassportError::EmptyRole,
        PassportError::InvalidRole,
        |ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
    )
}

/// Validates the zone field.
fn validate_zone(value: &str) -> Result<(), PassportError> {
    validate_canonical_text(
        value,
        MAX_ZONE_LEN,
        PassportError::EmptyZone,
        PassportError::InvalidZone,
        |ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-',
    )
}

/// Validates the embedded certificate field.
///
/// Compatibility note:
/// this remains string-based to preserve the current passport shape.
fn validate_certificate(value: &str) -> Result<(), PassportError> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(PassportError::EmptyCertificate);
    }

    if value != value.trim() {
        return Err(PassportError::InvalidCertificate);
    }

    if value.len() > MAX_CERTIFICATE_LEN {
        return Err(PassportError::InvalidCertificate);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passport_roundtrip() {
        let passport = Passport::new(
            "AOXC-VAL-EU-1234".into(),
            "validator".into(),
            "EU".into(),
            "CERT_DATA".into(),
            100,
            200,
        );

        let json = passport.to_json().unwrap();
        let restored = Passport::from_json(&json).unwrap();

        assert_eq!(passport.actor_id, restored.actor_id);
        assert_eq!(passport, restored);
    }

    #[test]
    fn expiration_check() {
        let passport = Passport::new(
            "AOXC-ACTOR-1".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        assert!(passport.is_expired(300));
        assert!(passport.is_expired(200));
        assert!(!passport.is_expired(150));
    }

    #[test]
    fn validity_state_helpers_work() {
        let passport = Passport::new(
            "AOXC-ACTOR-2".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        assert!(passport.is_not_yet_valid_at(99));
        assert!(passport.is_valid_at(150));
        assert_eq!(
            passport.validity_state_at(99),
            PassportValidityState::NotYetValid
        );
        assert_eq!(
            passport.validity_state_at(150),
            PassportValidityState::Valid
        );
        assert_eq!(
            passport.validity_state_at(200),
            PassportValidityState::Expired
        );
    }

    #[test]
    fn fingerprint_is_stable() {
        let passport = Passport::new(
            "AOXC-ACTOR-3".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        let a = passport.fingerprint();
        let b = passport.fingerprint();

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn certificate_fingerprint_is_stable() {
        let passport = Passport::new(
            "AOXC-ACTOR-4".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        let a = passport.certificate_fingerprint();
        let b = passport.certificate_fingerprint();

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn invalid_validity_window_is_rejected() {
        let passport = Passport::new(
            "AOXC-ACTOR-5".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            100,
        );

        assert_eq!(
            passport.validate(),
            Err(PassportError::InvalidValidityWindow)
        );
    }

    #[test]
    fn invalid_actor_id_is_rejected() {
        let passport = Passport::new(
            "bad actor".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        assert_eq!(passport.validate(), Err(PassportError::InvalidActorId));
    }

    #[test]
    fn surrounding_whitespace_is_rejected() {
        let passport = Passport::new(
            " AOXC-ACTOR-6 ".into(),
            "node".into(),
            "EU".into(),
            "cert".into(),
            100,
            200,
        );

        assert_eq!(passport.validate(), Err(PassportError::InvalidActorId));
    }

    #[test]
    fn from_json_rejects_invalid_passport_payload() {
        let data = r#"{
            "version": 1,
            "actor_id": "bad actor",
            "role": "node",
            "zone": "EU",
            "certificate": "cert",
            "issued_at": 100,
            "expires_at": 200
        }"#;

        let result = Passport::from_json(data);
        assert!(matches!(result, Err(error) if error.contains("PASSPORT_VALIDATE_ERROR")));
    }
}
