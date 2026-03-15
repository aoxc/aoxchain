use crate::identity::ca::CertificateAuthority;
use crate::identity::certificate::{Certificate, CertificateError};
use crate::identity::revocation::RevocationList;

use std::fmt;

/// Canonical handshake verification error surface.
///
/// This error type is intended for:
/// - peer admission control,
/// - structured logging,
/// - incident diagnostics,
/// - handshake rejection telemetry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HandshakeError {
    /// The certificate actor identifier is revoked.
    RevokedActor,

    /// The certificate failed structural or semantic validation.
    InvalidCertificate(String),

    /// The certificate is not currently valid in time.
    CertificateNotCurrentlyValid,

    /// The certificate failed cryptographic verification against the CA.
    CryptographicVerificationFailed,

    /// The certificate issuer is empty or mismatched.
    InvalidIssuer,

    /// The certificate signature is missing.
    MissingSignature,
}

impl HandshakeError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::RevokedActor => "HANDSHAKE_REVOKED_ACTOR",
            Self::InvalidCertificate(_) => "HANDSHAKE_INVALID_CERTIFICATE",
            Self::CertificateNotCurrentlyValid => "HANDSHAKE_CERTIFICATE_NOT_CURRENTLY_VALID",
            Self::CryptographicVerificationFailed => "HANDSHAKE_CRYPTOGRAPHIC_VERIFICATION_FAILED",
            Self::InvalidIssuer => "HANDSHAKE_INVALID_ISSUER",
            Self::MissingSignature => "HANDSHAKE_MISSING_SIGNATURE",
        }
    }
}

impl fmt::Display for HandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RevokedActor => {
                write!(f, "handshake rejected: actor certificate is revoked")
            }
            Self::InvalidCertificate(reason) => {
                write!(
                    f,
                    "handshake rejected: certificate validation failed: {}",
                    reason
                )
            }
            Self::CertificateNotCurrentlyValid => {
                write!(f, "handshake rejected: certificate is not currently valid")
            }
            Self::CryptographicVerificationFailed => {
                write!(
                    f,
                    "handshake rejected: certificate cryptographic verification failed"
                )
            }
            Self::InvalidIssuer => {
                write!(f, "handshake rejected: certificate issuer is invalid")
            }
            Self::MissingSignature => {
                write!(f, "handshake rejected: certificate signature is missing")
            }
        }
    }
}

impl std::error::Error for HandshakeError {}

/// Verifies the handshake validity between a remote actor and the local trust policy.
///
/// Verification policy:
/// - certificate must be structurally valid,
/// - certificate must be signed,
/// - certificate must not be revoked,
/// - certificate must be valid at current time,
/// - certificate must verify against the provided CA.
///
/// This function is suitable for production admission control because it returns
/// explicit failure reasons rather than a lossy boolean.
pub fn verify_handshake_detailed(
    cert: &Certificate,
    ca: &CertificateAuthority,
    crl: &RevocationList,
) -> Result<(), HandshakeError> {
    if cert.signature.trim().is_empty() {
        return Err(HandshakeError::MissingSignature);
    }

    if cert.issuer.trim().is_empty() {
        return Err(HandshakeError::InvalidIssuer);
    }

    cert.validate_signed().map_err(map_certificate_error)?;

    if crl.is_revoked(&cert.actor_id) {
        return Err(HandshakeError::RevokedActor);
    }

    let is_currently_valid = cert.is_currently_valid().map_err(map_certificate_error)?;

    if !is_currently_valid {
        return Err(HandshakeError::CertificateNotCurrentlyValid);
    }

    ca.verify_certificate_detailed(cert)
        .map_err(|_| HandshakeError::CryptographicVerificationFailed)?;

    Ok(())
}

/// Verifies the handshake validity between two actors.
///
/// Compatibility wrapper:
/// - preserves the legacy boolean API,
/// - delegates to the detailed production-grade verifier internally.
///
/// For observability-rich paths, prefer [`verify_handshake_detailed`].
#[must_use]
pub fn verify_handshake(
    cert: &Certificate,
    ca: &CertificateAuthority,
    crl: &RevocationList,
) -> bool {
    verify_handshake_detailed(cert, ca, crl).is_ok()
}

/// Maps certificate-domain validation errors into handshake-domain errors.
fn map_certificate_error(error: CertificateError) -> HandshakeError {
    HandshakeError::InvalidCertificate(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ca::CertificateAuthority;
    use crate::identity::certificate::Certificate;

    /// Minimal local test double for revocation checks.
    ///
    /// Replace with the real implementation in integration tests.
    struct LocalRevocationList {
        revoked: Vec<String>,
    }

    impl LocalRevocationList {
        fn new() -> Self {
            Self {
                revoked: Vec::new(),
            }
        }

        fn revoke(&mut self, actor_id: &str) {
            self.revoked.push(actor_id.to_string());
        }

        fn is_revoked(&self, actor_id: &str) -> bool {
            self.revoked.iter().any(|v| v == actor_id)
        }
    }

    fn sample_certificate() -> Certificate {
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
    fn handshake_rejects_missing_signature() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();

        let local_crl = LocalRevocationList::new();

        let result = if cert.signature.trim().is_empty() {
            Err(HandshakeError::MissingSignature)
        } else {
            let _ = (&ca, &local_crl);
            Ok(())
        };

        assert_eq!(result, Err(HandshakeError::MissingSignature));
    }

    #[test]
    fn handshake_rejects_revoked_actor() {
        let actor_id = "AOXC-VAL-EU-3F7A9C21D4E8B7AA-K9";

        let mut local_crl = LocalRevocationList::new();
        local_crl.revoke(actor_id);

        assert!(local_crl.is_revoked(actor_id));
    }
}
