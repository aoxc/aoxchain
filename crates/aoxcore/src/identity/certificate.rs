use serde::{Deserialize, Serialize};
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
    pub fn signing_payload_bytes(&self) -> Result<Vec<u8>, CertificateError> {
        serde_json::to_vec(&self.signing_payload())
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

        if self.issued_at == 0 {
            return Err(CertificateError::InvalidIssuedAt);
        }

        if self.expires_at == 0 {
            return Err(CertificateError::InvalidExpiresAt);
        }

        if self.expires_at <= self.issued_at {
            return Err(CertificateError::InvalidValidityWindow);
        }

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

        if self.signature.trim().is_empty() {
            return Err(CertificateError::EmptySignature);
        }

        if !is_valid_upper_or_lower_hex(self.signature.trim()) {
            return Err(CertificateError::InvalidSignatureHex);
        }

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
}

/// Returns the current UNIX timestamp in seconds.
fn current_unix_time() -> Result<u64, CertificateError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| CertificateError::TimeError)
}

/// Validates the chain field.
fn validate_chain(value: &str) -> Result<(), CertificateError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyChain);
    }

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
fn validate_actor_id(value: &str) -> Result<(), CertificateError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyActorId);
    }

    if trimmed.len() > MAX_ACTOR_ID_LEN {
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
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyRole);
    }

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
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyZone);
    }

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
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyIssuer);
    }

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

/// Validates the public key hex field.
fn validate_pubkey_hex(value: &str) -> Result<(), CertificateError> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err(CertificateError::EmptyPublicKey);
    }

    if trimmed.len() > MAX_PUBKEY_HEX_LEN || !trimmed.len().is_multiple_of(2) {
        return Err(CertificateError::InvalidPublicKeyHex);
    }

    if !is_valid_upper_or_lower_hex(trimmed) {
        return Err(CertificateError::InvalidPublicKeyHex);
    }

    Ok(())
}

/// Returns true if the provided string is valid hexadecimal.
fn is_valid_upper_or_lower_hex(value: &str) -> bool {
    value.chars().all(|ch| ch.is_ascii_hexdigit())
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
    fn signing_payload_excludes_signature() {
        let mut cert = sample_unsigned();
        cert.issuer = "AOXC-ROOT-CA".to_string();
        cert.signature = "DEADBEEF".to_string();

        let payload = cert.signing_payload();
        assert_eq!(payload.issuer, "AOXC-ROOT-CA");
        assert_eq!(payload.actor_id, cert.actor_id);
    }

    #[test]
    fn unsigned_view_clears_signature() {
        let mut cert = sample_unsigned();
        cert.signature = "BEEF".to_string();

        let unsigned = cert.unsigned_view();
        assert!(unsigned.signature.is_empty());
    }

    #[test]
    fn validity_helpers_work() {
        let cert = sample_unsigned();

        assert!(cert.is_valid_at(1_750_000_000));
        assert!(cert.is_expired_at(1_800_000_000));
        assert!(cert.is_not_yet_valid_at(1_600_000_000));
    }
}
