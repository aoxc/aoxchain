// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Current certificate schema version supported by the AOXC identity layer.
pub const CERTIFICATE_VERSION: u8 = 1;

/// Maximum accepted chain identifier length.
const MAX_CHAIN_LEN: usize = 64;

/// Maximum accepted actor identifier length.
const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum accepted role length.
const MAX_ROLE_LEN: usize = 32;

/// Maximum accepted zone length.
const MAX_ZONE_LEN: usize = 32;

/// Maximum accepted issuer length.
const MAX_ISSUER_LEN: usize = 128;

/// Maximum accepted public-key hex length.
///
/// This bound is intentionally generous to support larger post-quantum public keys.
const MAX_PUBKEY_HEX_LEN: usize = 8192;

/// Maximum accepted detached-signature hex length.
///
/// This bound is intentionally generous enough for large post-quantum signature
/// surfaces while still rejecting obviously malformed or unbounded payloads.
const MAX_SIGNATURE_HEX_LEN: usize = 16384;

/// Canonical domain separator for certificate fingerprints.
///
/// Security rationale:
/// - provides explicit namespace separation for operator-facing digest helpers,
/// - avoids accidental cross-domain reuse of the same serialized bytes.
const CERTIFICATE_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/CERTIFICATE/FINGERPRINT/V1";

/// Canonical certificate payload used for signing.
///
/// The signature field is intentionally excluded from this structure so that:
/// - signing input is deterministic,
/// - verification input is stable,
/// - domain-specific signing code can serialize this payload directly.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CertificateSigningPayload {
    pub version: u8,
    pub chain: String,
    pub actor_id: String,
    pub role: String,
    pub zone: String,
    pub pubkey: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub issuer: String,
}

/// Canonical certificate object used by the AOXC identity layer.
///
/// Compatibility notes:
/// - Field names are preserved exactly as provided.
/// - `issuer` and `signature` remain plain strings for compatibility with the
///   existing CA and persistence model.
/// - Validation and signing helpers are added without breaking the data shape.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    pub version: u8,
    pub chain: String,
    pub actor_id: String,
    pub role: String,
    pub zone: String,
    pub pubkey: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub issuer: String,
    pub signature: String,
}

/// Current lifecycle classification for a certificate at a specific timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertificateValidityState {
    NotYetValid,
    Valid,
    Expired,
}

/// Canonical error type for certificate-domain validation and lifecycle checks.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CertificateError {
    InvalidVersion,
    EmptyChain,
    InvalidChain,
    EmptyActorId,
    InvalidActorId,
    EmptyRole,
    InvalidRole,
    EmptyZone,
    InvalidZone,
    EmptyPublicKey,
    InvalidPublicKeyHex,
    InvalidIssuedAt,
    InvalidExpiresAt,
    InvalidValidityWindow,
    EmptyIssuer,
    InvalidIssuer,
    EmptySignature,
    InvalidSignatureHex,
    SerializationFailed(String),
    TimeError,
}

impl CertificateError {
    /// Returns a stable symbolic error code suitable for logging and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidVersion => "CERT_INVALID_VERSION",
            Self::EmptyChain => "CERT_EMPTY_CHAIN",
            Self::InvalidChain => "CERT_INVALID_CHAIN",
            Self::EmptyActorId => "CERT_EMPTY_ACTOR_ID",
            Self::InvalidActorId => "CERT_INVALID_ACTOR_ID",
            Self::EmptyRole => "CERT_EMPTY_ROLE",
            Self::InvalidRole => "CERT_INVALID_ROLE",
            Self::EmptyZone => "CERT_EMPTY_ZONE",
            Self::InvalidZone => "CERT_INVALID_ZONE",
            Self::EmptyPublicKey => "CERT_EMPTY_PUBLIC_KEY",
            Self::InvalidPublicKeyHex => "CERT_INVALID_PUBLIC_KEY_HEX",
            Self::InvalidIssuedAt => "CERT_INVALID_ISSUED_AT",
            Self::InvalidExpiresAt => "CERT_INVALID_EXPIRES_AT",
            Self::InvalidValidityWindow => "CERT_INVALID_VALIDITY_WINDOW",
            Self::EmptyIssuer => "CERT_EMPTY_ISSUER",
            Self::InvalidIssuer => "CERT_INVALID_ISSUER",
            Self::EmptySignature => "CERT_EMPTY_SIGNATURE",
            Self::InvalidSignatureHex => "CERT_INVALID_SIGNATURE_HEX",
            Self::SerializationFailed(_) => "CERT_SERIALIZATION_FAILED",
            Self::TimeError => "CERT_TIME_ERROR",
        }
    }
}

impl fmt::Display for CertificateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion => write!(f, "certificate validation failed: unsupported version"),
            Self::EmptyChain => write!(f, "certificate validation failed: chain must not be empty"),
            Self::InvalidChain => {
                write!(f, "certificate validation failed: chain format is invalid")
            }
            Self::EmptyActorId => write!(
                f,
                "certificate validation failed: actor_id must not be empty"
            ),
            Self::InvalidActorId => write!(
                f,
                "certificate validation failed: actor_id format is invalid"
            ),
            Self::EmptyRole => write!(f, "certificate validation failed: role must not be empty"),
            Self::InvalidRole => write!(f, "certificate validation failed: role format is invalid"),
            Self::EmptyZone => write!(f, "certificate validation failed: zone must not be empty"),
            Self::InvalidZone => write!(f, "certificate validation failed: zone format is invalid"),
            Self::EmptyPublicKey => write!(
                f,
                "certificate validation failed: public key must not be empty"
            ),
            Self::InvalidPublicKeyHex => write!(
                f,
                "certificate validation failed: public key must be valid hexadecimal"
            ),
            Self::InvalidIssuedAt => {
                write!(f, "certificate validation failed: issued_at is invalid")
            }
            Self::InvalidExpiresAt => {
                write!(f, "certificate validation failed: expires_at is invalid")
            }
            Self::InvalidValidityWindow => write!(
                f,
                "certificate validation failed: expires_at must be greater than issued_at"
            ),
            Self::EmptyIssuer => {
                write!(f, "certificate validation failed: issuer must not be empty")
            }
            Self::InvalidIssuer => {
                write!(f, "certificate validation failed: issuer format is invalid")
            }
            Self::EmptySignature => write!(
                f,
                "certificate validation failed: signature must not be empty"
            ),
            Self::InvalidSignatureHex => write!(
                f,
                "certificate validation failed: signature must be valid hexadecimal"
            ),
            Self::SerializationFailed(error) => {
                write!(f, "certificate serialization failed: {}", error)
            }
            Self::TimeError => write!(f, "certificate time check failed: system time is invalid"),
        }
    }
}

impl std::error::Error for CertificateError {}

impl Default for Certificate {
    fn default() -> Self {
        Self::new_unsigned(
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            0,
            0,
        )
    }
}

impl CertificateSigningPayload {
    /// Validates the canonical certificate signing payload.
    ///
    /// This helper is useful in cases where callers need to validate the exact
    /// object that will be serialized and signed.
    pub fn validate(&self) -> Result<(), CertificateError> {
        if self.version != CERTIFICATE_VERSION {
            return Err(CertificateError::InvalidVersion);
        }

        validate_chain(&self.chain)?;
        validate_actor_id(&self.actor_id)?;
        validate_role(&self.role)?;
        validate_zone(&self.zone)?;
        validate_pubkey_hex(&self.pubkey)?;
        validate_issuer(&self.issuer)?;

        validate_validity_window(self.issued_at, self.expires_at)
    }
}

impl Certificate {
    /// Creates a new unsigned certificate.
    ///
    /// The `issuer` and `signature` fields are initialized empty so that a CA
    /// may later canonicalize and sign the certificate.
    #[must_use]
    pub fn new_unsigned(
        chain: String,
        actor_id: String,
        role: String,
        zone: String,
        pubkey: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Self {
        Self {
            version: CERTIFICATE_VERSION,
            chain,
            actor_id,
            role,
            zone,
            pubkey,
            issued_at,
            expires_at,
            issuer: String::new(),
            signature: String::new(),
        }
    }

    /// Returns the canonical signing payload for this certificate.
    ///
    /// The returned payload excludes the detached signature field.
    #[must_use]
    pub fn signing_payload(&self) -> CertificateSigningPayload {
        CertificateSigningPayload {
            version: self.version,
            chain: self.chain.clone(),
            actor_id: self.actor_id.clone(),
            role: self.role.clone(),
            zone: self.zone.clone(),
            pubkey: self.pubkey.clone(),
            issued_at: self.issued_at,
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
        }
    }

    /// Serializes the canonical signing payload into JSON bytes.
    ///
    /// The payload is validated before serialization so that callers cannot
    /// accidentally sign semantically invalid certificate content.
    pub fn signing_payload_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        let payload = self.signing_payload();
        payload.validate()?;

        serde_json::to_vec(&payload)
            .map_err(|error| CertificateError::SerializationFailed(error.to_string()))
    }

    /// Returns a copy of the certificate with the signature field cleared.
    #[must_use]
    pub fn unsigned_view(&self) -> Self {
        let mut cloned = self.clone();
        cloned.signature.clear();
        cloned
    }

    /// Returns true if the certificate currently carries a signature.
    #[must_use]
    pub fn is_signed(&self) -> bool {
        !self.signature.trim().is_empty()
    }

    /// Returns true if the certificate issuer is present.
    #[must_use]
    pub fn has_issuer(&self) -> bool {
        !self.issuer.trim().is_empty()
    }

    /// Returns the decoded public key bytes.
    pub fn public_key_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        validate_pubkey_hex(&self.pubkey)?;
        hex::decode(self.pubkey.trim()).map_err(|_| CertificateError::InvalidPublicKeyHex)
    }

    /// Returns the decoded detached signature bytes.
    pub fn signature_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        validate_signature_hex(&self.signature)?;
        hex::decode(self.signature.trim()).map_err(|_| CertificateError::InvalidSignatureHex)
    }

    /// Returns a deterministic operator-facing fingerprint of the certificate.
    ///
    /// The fingerprint is derived from the full serialized certificate object,
    /// including issuer and signature fields.
    pub fn fingerprint(&self) -> Result<String, CertificateError> {
        let body = serde_json::to_vec(self)
            .map_err(|error| CertificateError::SerializationFailed(error.to_string()))?;

        let mut hasher = Sha3_256::new();
        hasher.update(CERTIFICATE_FINGERPRINT_DOMAIN);
        hasher.update([0x00]);
        hasher.update(body);

        let digest = hasher.finalize();
        Ok(hex::encode_upper(&digest[..8]))
    }

    /// Returns the current validity state at the supplied UNIX timestamp.
    #[must_use]
    pub fn validity_state_at(&self, unix_time: u64) -> CertificateValidityState {
        if self.is_not_yet_valid_at(unix_time) {
            CertificateValidityState::NotYetValid
        } else if self.is_expired_at(unix_time) {
            CertificateValidityState::Expired
        } else {
            CertificateValidityState::Valid
        }
    }

    /// Validates the certificate fields required before signing.
    ///
    /// This validation intentionally does not require `issuer` or `signature`
    /// to be populated, because unsigned certificates are valid intermediate
    /// objects during issuance workflows.
    pub fn validate_unsigned(&self) -> Result<(), CertificateError> {
        if self.version != CERTIFICATE_VERSION {
            return Err(CertificateError::InvalidVersion);
        }

        validate_chain(&self.chain)?;
        validate_actor_id(&self.actor_id)?;
        validate_role(&self.role)?;
        validate_zone(&self.zone)?;
        validate_pubkey_hex(&self.pubkey)?;
        validate_validity_window(self.issued_at, self.expires_at)?;

        Ok(())
    }

    /// Validates the fully issued certificate.
    ///
    /// This includes:
    /// - unsigned payload validation,
    /// - issuer validation,
    /// - signature presence and encoding validation.
    pub fn validate_signed(&self) -> Result<(), CertificateError> {
        self.validate_unsigned()?;
        validate_issuer(&self.issuer)?;
        validate_signature_hex(&self.signature)?;
        Ok(())
    }

    /// Returns true if the certificate is valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_valid_at(&self, unix_time: u64) -> bool {
        unix_time >= self.issued_at && unix_time < self.expires_at
    }

    /// Returns true if the certificate is expired at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_expired_at(&self, unix_time: u64) -> bool {
        unix_time >= self.expires_at
    }

    /// Returns true if the certificate is not yet valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_not_yet_valid_at(&self, unix_time: u64) -> bool {
        unix_time < self.issued_at
    }

    /// Returns true if the certificate is currently valid according to system time.
    pub fn is_currently_valid(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_valid_at(now))
    }

    /// Returns true if the certificate is currently expired according to system time.
    pub fn is_currently_expired(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_expired_at(now))
    }

    /// Returns true if the certificate is currently not yet valid according to system time.
    pub fn is_currently_not_yet_valid(&self) -> Result<bool, CertificateError> {
        let now = current_unix_time()?;
        Ok(self.is_not_yet_valid_at(now))
    }
}

/// Returns the current UNIX timestamp in seconds.
fn current_unix_time() -> Result<u64, CertificateError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CertificateError::TimeError)
}

/// Validates the shared issued/expires window.
fn validate_validity_window(issued_at: u64, expires_at: u64) -> Result<(), CertificateError> {
    if issued_at == 0 {
        return Err(CertificateError::InvalidIssuedAt);
    }

    if expires_at == 0 {
        return Err(CertificateError::InvalidExpiresAt);
    }

    if expires_at <= issued_at {
        return Err(CertificateError::InvalidValidityWindow);
    }

    Ok(())
}

/// Validates that a field does not contain surrounding whitespace and is not blank.
fn validate_canonical_text_presence(
    value: &str,
    empty_error: CertificateError,
    invalid_error: CertificateError,
) -> Result<&str, CertificateError> {
    if value.is_empty() {
        return Err(empty_error);
    }

    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(empty_error);
    }

    if trimmed != value {
        return Err(invalid_error);
    }

    Ok(trimmed)
}

/// Validates the chain field.
fn validate_chain(value: &str) -> Result<(), CertificateError> {
    let trimmed =
        validate_canonical_text_presence(value, CertificateError::EmptyChain, CertificateError::InvalidChain)?;

    if trimmed.len() > MAX_CHAIN_LEN {
        return Err(CertificateError::InvalidChain);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidChain);
    }

    Ok(())
}

/// Validates the actor_id field.
///
/// Compatibility note:
/// this validator preserves a string-based certificate-layer contract rather
/// than importing a stricter actor-id parser directly. That keeps the
/// certificate object tolerant of legacy yet still bounded AOXC actor-id
/// representations while rejecting malformed input.
fn validate_actor_id(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyActorId,
        CertificateError::InvalidActorId,
    )?;

    if trimmed.len() > MAX_ACTOR_ID_LEN {
        return Err(CertificateError::InvalidActorId);
    }

    if !trimmed.starts_with("AOXC-") {
        return Err(CertificateError::InvalidActorId);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidActorId);
    }

    Ok(())
}

/// Validates the role field.
fn validate_role(value: &str) -> Result<(), CertificateError> {
    let trimmed =
        validate_canonical_text_presence(value, CertificateError::EmptyRole, CertificateError::InvalidRole)?;

    if trimmed.len() > MAX_ROLE_LEN {
        return Err(CertificateError::InvalidRole);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(CertificateError::InvalidRole);
    }

    Ok(())
}

/// Validates the zone field.
fn validate_zone(value: &str) -> Result<(), CertificateError> {
    let trimmed =
        validate_canonical_text_presence(value, CertificateError::EmptyZone, CertificateError::InvalidZone)?;

    if trimmed.len() > MAX_ZONE_LEN {
        return Err(CertificateError::InvalidZone);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        return Err(CertificateError::InvalidZone);
    }

    Ok(())
}

/// Validates the issuer field.
fn validate_issuer(value: &str) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(
        value,
        CertificateError::EmptyIssuer,
        CertificateError::InvalidIssuer,
    )?;

    if trimmed.len() > MAX_ISSUER_LEN {
        return Err(CertificateError::InvalidIssuer);
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(CertificateError::InvalidIssuer);
    }

    Ok(())
}

/// Validates a hexadecimal field with bounded length and even-width encoding.
fn validate_hex_field(
    value: &str,
    empty_error: CertificateError,
    invalid_error: CertificateError,
    max_len: usize,
) -> Result<(), CertificateError> {
    let trimmed = validate_canonical_text_presence(value, empty_error, invalid_error)?;

    if trimmed.len() > max_len || trimmed.len() % 2 != 0 {
        return Err(invalid_error);
    }

    if !trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(invalid_error);
    }

    Ok(())
}

/// Validates the public key hex field.
fn validate_pubkey_hex(value: &str) -> Result<(), CertificateError> {
    validate_hex_field(
        value,
        CertificateError::EmptyPublicKey,
        CertificateError::InvalidPublicKeyHex,
        MAX_PUBKEY_HEX_LEN,
    )
}

/// Validates the signature hex field.
fn validate_signature_hex(value: &str) -> Result<(), CertificateError> {
    validate_hex_field(
        value,
        CertificateError::EmptySignature,
        CertificateError::InvalidSignatureHex,
        MAX_SIGNATURE_HEX_LEN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_unsigned() -> Certificate {
        Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            1_700_000_000,
            1_800_000_000,
        )
    }

    #[test]
    fn unsigned_certificate_validates_successfully() {
        let cert = sample_unsigned();
        assert_eq!(cert.validate_unsigned(), Ok(()));
    }

    #[test]
    fn signed_certificate_requires_issuer_and_signature() {
        let cert = sample_unsigned();
        assert_eq!(cert.validate_signed(), Err(CertificateError::EmptyIssuer));
    }

    #[test]
    fn signed_certificate_validates_when_completed() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "A1B2".to_string();

        assert_eq!(cert.validate_signed(), Ok(()));
    }

    #[test]
    fn invalid_validity_window_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            100,
            100,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidValidityWindow)
        );
    }

    #[test]
    fn invalid_public_key_hex_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "ZZ_NOT_HEX".to_string(),
            100,
            200,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidPublicKeyHex)
        );
    }

    #[test]
    fn odd_length_public_key_hex_is_rejected() {
        let cert = Certificate::new_unsigned(
            "AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "ABC".to_string(),
            100,
            200,
        );

        assert_eq!(
            cert.validate_unsigned(),
            Err(CertificateError::InvalidPublicKeyHex)
        );
    }

    #[test]
    fn odd_length_signature_hex_is_rejected() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "ABC".to_string();

        assert_eq!(
            cert.validate_signed(),
            Err(CertificateError::InvalidSignatureHex)
        );
    }

    #[test]
    fn surrounding_whitespace_is_rejected() {
        let cert = Certificate::new_unsigned(
            " AOXC-0001-MAIN".to_string(),
            "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9".to_string(),
            "VAL".to_string(),
            "EU".to_string(),
            "A1B2C3D4".to_string(),
            1_700_000_000,
            1_800_000_000,
        );

        assert_eq!(cert.validate_unsigned(), Err(CertificateError::InvalidChain));
    }

    #[test]
    fn signing_payload_excludes_signature() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let payload = cert.signing_payload();
        assert_eq!(payload.issuer, "AOXC-ROOT-CA");
        assert_eq!(payload.actor_id, cert.actor_id);
    }

    #[test]
    fn signing_payload_bytes_validate_before_serialization() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();

        let bytes = cert
            .signing_payload_bytes()
            .expect("signing payload serialization must succeed");

        assert!(!bytes.is_empty());
    }

    #[test]
    fn unsigned_view_clears_signature() {
        let mut cert = sample_unsigned();
        cert.signature = "BEEF".to_string();

        let unsigned = cert.unsigned_view();
        assert!(unsigned.signature.is_empty());
    }

    #[test]
    fn decoded_public_key_bytes_are_available() {
        let cert = sample_unsigned();

        let decoded = cert
            .public_key_bytes()
            .expect("public key bytes must decode successfully");

        assert_eq!(decoded, vec![0xA1, 0xB2, 0xC3, 0xD4]);
    }

    #[test]
    fn decoded_signature_bytes_are_available() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let decoded = cert
            .signature_bytes()
            .expect("signature bytes must decode successfully");

        assert_eq!(decoded, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn fingerprint_is_stable() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let a = cert.fingerprint().expect("fingerprint must succeed");
        let b = cert.fingerprint().expect("fingerprint must succeed");

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn validity_helpers_work() {
        let cert = sample_unsigned();

        assert!(cert.is_valid_at(1_750_000_000));
        assert!(cert.is_expired_at(1_800_000_000));
        assert!(cert.is_not_yet_valid_at(1_600_000_000));
        assert_eq!(
            cert.validity_state_at(1_750_000_000),
            CertificateValidityState::Valid
        );
        assert_eq!(
            cert.validity_state_at(1_600_000_000),
            CertificateValidityState::NotYetValid
        );
        assert_eq!(
            cert.validity_state_at(1_800_000_000),
            CertificateValidityState::Expired
        );
    }
}
