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
/// This upper bound is intentionally generous to accommodate large
/// post-quantum public-key representations while still enforcing a finite
/// and reviewable input surface.
const MAX_PUBKEY_HEX_LEN: usize = 8192;

/// Maximum accepted detached-signature hex length.
///
/// This upper bound is intentionally generous for large post-quantum
/// signature material while still rejecting obviously malformed or
/// unbounded payloads.
const MAX_SIGNATURE_HEX_LEN: usize = 16384;

/// Canonical domain separator for certificate fingerprints.
///
/// Security rationale:
/// - provides explicit namespace separation for operator-facing digest helpers,
/// - prevents accidental cross-domain reuse of the same serialized bytes.
const CERTIFICATE_FINGERPRINT_DOMAIN: &[u8] = b"AOXC/IDENTITY/CERTIFICATE/FINGERPRINT/V1";

/// Canonical certificate payload used for signing.
///
/// The detached signature field is intentionally excluded so that:
/// - signing input remains deterministic,
/// - verification input remains stable,
/// - domain-specific signing code can serialize this object directly.
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
/// - field names are preserved exactly as stored and serialized,
/// - `issuer` and `signature` remain plain strings for compatibility with the
///   current certificate authority and persistence model,
/// - validation and signing helpers extend behavior without changing shape.
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

/// Lifecycle classification for a certificate at a specific timestamp.
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
    /// Returns a stable symbolic error code suitable for logs, metrics,
    /// incident correlation, and operator telemetry.
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
            Self::InvalidVersion => {
                write!(f, "certificate validation failed: unsupported version")
            }
            Self::EmptyChain => {
                write!(f, "certificate validation failed: chain must not be empty")
            }
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
            Self::EmptyRole => {
                write!(f, "certificate validation failed: role must not be empty")
            }
            Self::InvalidRole => {
                write!(f, "certificate validation failed: role format is invalid")
            }
            Self::EmptyZone => {
                write!(f, "certificate validation failed: zone must not be empty")
            }
            Self::InvalidZone => {
                write!(f, "certificate validation failed: zone format is invalid")
            }
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
            Self::TimeError => {
                write!(f, "certificate time check failed: system time is invalid")
            }
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
    /// This helper is intended for workflows that must validate the exact
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
        validate_validity_window(self.issued_at, self.expires_at)?;

        Ok(())
    }
}

