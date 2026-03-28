// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

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
    ///
    /// The enclosed string is intended to carry a stable symbolic reason code.
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
/// - certificate issuer must be present and match the supplied CA issuer,
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
    verify_handshake_common(cert, ca, crl)?;

    let is_currently_valid = cert.is_currently_valid().map_err(map_certificate_error)?;

    if !is_currently_valid {
        return Err(HandshakeError::CertificateNotCurrentlyValid);
    }

    Ok(())
}

/// Verifies the handshake validity at an explicit UNIX timestamp.
///
/// This helper is intended for:
/// - deterministic tests,
/// - replayed incident analysis,
/// - time-travel validation,
/// - simulation environments.
pub fn verify_handshake_detailed_at(
    cert: &Certificate,
    ca: &CertificateAuthority,
    crl: &RevocationList,
    unix_time: u64,
) -> Result<(), HandshakeError> {
    verify_handshake_common(cert, ca, crl)?;

    if !cert.is_valid_at(unix_time) {
        return Err(HandshakeError::CertificateNotCurrentlyValid);
    }

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

/// Applies all handshake checks except current-time evaluation.
///
/// Separation rationale:
/// - keeps the current-time path and explicit-time path aligned,
/// - avoids duplicating cryptographic and policy validation logic.
fn verify_handshake_common(
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

    if ca.issuer.trim().is_empty() {
        return Err(HandshakeError::InvalidIssuer);
    }

    if cert.issuer != ca.issuer {
        return Err(HandshakeError::InvalidIssuer);
    }

    cert.validate_signed().map_err(map_certificate_error)?;

    if crl.is_revoked(&cert.actor_id) {
        return Err(HandshakeError::RevokedActor);
    }

    ca.verify_certificate_detailed(cert)
        .map_err(|_| HandshakeError::CryptographicVerificationFailed)?;

    Ok(())
}

/// Maps certificate-domain validation errors into handshake-domain errors.
///
/// Mapping policy:
/// - missing signature remains a first-class handshake error,
/// - issuer problems remain a first-class handshake error,
/// - all other certificate-domain validation failures are collapsed into
///   `InvalidCertificate` with a stable symbolic reason code.
fn map_certificate_error(error: CertificateError) -> HandshakeError {
    match error {
        CertificateError::EmptySignature => HandshakeError::MissingSignature,
        CertificateError::EmptyIssuer | CertificateError::InvalidIssuer => {
            HandshakeError::InvalidIssuer
        }
        other => HandshakeError::InvalidCertificate(other.code().to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::revocation::{RevocationList, RevocationReason};

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

    fn sample_signed_certificate(ca: &CertificateAuthority) -> Certificate {
        ca.sign_certificate(sample_certificate())
            .expect("certificate signing must succeed")
    }

    #[test]
    fn handshake_accepts_valid_signed_certificate() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&ca);
        let crl = RevocationList::new();

        let result = verify_handshake_detailed(&cert, &ca, &crl);

        assert!(result.is_ok());
    }

    #[test]
    fn handshake_boolean_wrapper_matches_detailed_success_path() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&ca);
        let crl = RevocationList::new();

        assert!(verify_handshake(&cert, &ca, &crl));
    }

    #[test]
    fn handshake_rejects_missing_signature() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();
        let crl = RevocationList::new();

        let result = verify_handshake_detailed(&cert, &ca, &crl);

        assert_eq!(result, Err(HandshakeError::MissingSignature));
    }

    #[test]
    fn handshake_rejects_empty_issuer() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let mut cert = sample_certificate();
        cert.signature = "A1B2".to_string();

        let crl = RevocationList::new();
        let result = verify_handshake_detailed(&cert, &ca, &crl);

        assert_eq!(result, Err(HandshakeError::InvalidIssuer));
    }

    #[test]
    fn handshake_rejects_issuer_mismatch_before_crypto_verification() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let mut cert = sample_signed_certificate(&ca);
        cert.issuer = "OTHER-CA".to_string();

        let crl = RevocationList::new();
        let result = verify_handshake_detailed(&cert, &ca, &crl);

        assert_eq!(result, Err(HandshakeError::InvalidIssuer));
    }

    #[test]
    fn handshake_rejects_revoked_actor() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&ca);

        let mut crl = RevocationList::new();
        crl.revoke(&cert.actor_id, RevocationReason::KeyCompromise);

        let result = verify_handshake_detailed(&cert, &ca, &crl);

        assert_eq!(result, Err(HandshakeError::RevokedActor));
    }

    #[test]
    fn handshake_rejects_cryptographic_verification_failure() {
        let signing_ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let verifying_ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&signing_ca);
        let crl = RevocationList::new();

        let result = verify_handshake_detailed(&cert, &verifying_ca, &crl);

        assert_eq!(
            result,
            Err(HandshakeError::CryptographicVerificationFailed)
        );
    }

    #[test]
    fn handshake_rejects_certificate_not_currently_valid_at_explicit_time() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&ca);
        let crl = RevocationList::new();

        let result = verify_handshake_detailed_at(&cert, &ca, &crl, 1_900_000_000);

        assert_eq!(result, Err(HandshakeError::CertificateNotCurrentlyValid));
    }

    #[test]
    fn handshake_detailed_at_accepts_valid_explicit_time() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_signed_certificate(&ca);
        let crl = RevocationList::new();

        let result = verify_handshake_detailed_at(&cert, &ca, &crl, 1_750_000_000);

        assert!(result.is_ok());
    }

    #[test]
    fn certificate_domain_errors_map_to_stable_handshake_errors() {
        assert_eq!(
            map_certificate_error(CertificateError::EmptySignature),
            HandshakeError::MissingSignature
        );
        assert_eq!(
            map_certificate_error(CertificateError::InvalidIssuer),
            HandshakeError::InvalidIssuer
        );
        assert_eq!(
            map_certificate_error(CertificateError::InvalidPublicKeyHex),
            HandshakeError::InvalidCertificate("CERT_INVALID_PUBLIC_KEY_HEX".to_string())
        );
    }
}
