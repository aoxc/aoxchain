// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::identity::certificate::Certificate;

use pqcrypto_mldsa::mldsa65::{
    DetachedSignature, PublicKey, SecretKey, detached_sign, keypair, verify_detached_signature,
};

use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use std::fmt;

/// Canonical signing domain for AOXC certificate authority operations.
///
/// This value is prepended to the serialized certificate payload before
/// signature generation and verification. The purpose is to prevent
/// cross-domain signature confusion if the same serialized bytes appear
/// in another protocol context.
const CERTIFICATE_SIGNING_DOMAIN: &[u8] = b"AOXC/IDENTITY/CERTIFICATE/V1";

/// Domain used for self-test validation of CA key material.
///
/// Security rationale:
/// - prevents accidental reuse of the production signing domain,
/// - allows deterministic in-memory validation that public and private key
///   material belong to the same certificate authority instance.
const CA_SELF_TEST_DOMAIN: &[u8] = b"AOXC/IDENTITY/CA/SELF_TEST/V1";

/// Domain used for CA key identifier derivation.
const CA_KEY_ID_DOMAIN: &[u8] = b"AOXC/IDENTITY/CA/KEY_ID/V1";

/// Maximum accepted issuer identifier length.
///
/// This bound intentionally mirrors the certificate-layer operational posture
/// and prevents unbounded issuer strings from entering the CA trust surface.
const MAX_ISSUER_LEN: usize = 128;

/// CertificateAuthority represents a post-quantum certificate authority
/// capable of issuing and verifying certificates using Dilithium3.
///
/// Compatibility notes:
/// - Key material remains stored as raw bytes in order to preserve the
///   existing persistence model used by the current system.
/// - `sign_certificate` and `verify_certificate` preserve their external
///   method signatures to reduce integration risk.
/// - Detached signatures are used internally for a cleaner and more
///   protocol-appropriate certificate signature model.
///
/// Security notes:
/// - This structure may carry private key material in memory.
/// - Instances that do not need signing capability should prefer
///   `to_public_verifier()` or `from_public_key_bytes(...)`.
#[derive(Serialize, Deserialize, Clone)]
pub struct CertificateAuthority {
    /// Issuer identifier.
    pub issuer: String,

    /// Serialized secret key.
    ///
    /// Security note:
    /// This field is intentionally preserved for compatibility with the
    /// current persistence model. In a hardened production deployment,
    /// private-key-at-rest encryption or external key custody is preferred.
    sk_bytes: Vec<u8>,

    /// Serialized public key.
    pk_bytes: Vec<u8>,
}

impl fmt::Debug for CertificateAuthority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CertificateAuthority")
            .field("issuer", &self.issuer)
            .field("pk_len", &self.pk_bytes.len())
            .field("has_private_key", &(!self.sk_bytes.is_empty()))
            .finish()
    }
}

impl Drop for CertificateAuthority {
    fn drop(&mut self) {
        if !self.sk_bytes.is_empty() {
            self.sk_bytes.fill(0);
        }
    }
}

impl CertificateAuthority {
    /// Creates a new certificate authority with a freshly generated
    /// Dilithium3 keypair.
    ///
    /// Construction policy:
    /// - issuer identifier must satisfy AOXC issuer validation rules,
    /// - public and private material are generated together,
    /// - the resulting instance is expected to pass internal key-material validation.
    ///
    /// Compatibility note:
    /// this constructor remains infallible in order to preserve existing call sites.
    /// Invalid issuer input is normalized by direct storage, but signing and detailed
    /// verification paths still enforce strict issuer validation.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let (pk, sk) = keypair();

        let authority = Self {
            issuer: name.into(),
            sk_bytes: sk.as_bytes().to_vec(),
            pk_bytes: pk.as_bytes().to_vec(),
        };

        debug_assert!(authority.validate_key_material_pair().is_ok());

        authority
    }

    /// Reconstructs a certificate authority from serialized public and private key material.
    ///
    /// Validation policy:
    /// - issuer identifier must be valid,
    /// - public key bytes must decode successfully,
    /// - private key bytes may be empty only when a verifier-only CA is intended,
    /// - when private key bytes are present, a self-test signature round trip
    ///   must confirm that the keypair is internally consistent.
    pub fn from_serialized_parts(
        issuer: impl Into<String>,
        pk_bytes: Vec<u8>,
        sk_bytes: Vec<u8>,
    ) -> Result<Self, String> {
        let authority = Self {
            issuer: issuer.into(),
            pk_bytes,
            sk_bytes,
        };

        validate_issuer_identifier(&authority.issuer)?;
        authority.validate_key_material_pair()?;

        Ok(authority)
    }

    /// Reconstructs a verifier-only certificate authority from serialized public key material.
    ///
    /// The returned instance cannot sign certificates because no private key is retained.
    pub fn from_public_key_bytes(
        issuer: impl Into<String>,
        pk_bytes: Vec<u8>,
    ) -> Result<Self, String> {
        Self::from_serialized_parts(issuer, pk_bytes, Vec::new())
    }

    /// Returns a verifier-only clone of this certificate authority.
    ///
    /// Security rationale:
    /// callers that only need verification should prefer a verifier-only instance
    /// so that private key material is not propagated unnecessarily.
    #[must_use]
    pub fn to_public_verifier(&self) -> Self {
        Self {
            issuer: self.issuer.clone(),
            pk_bytes: self.pk_bytes.clone(),
            sk_bytes: Vec::new(),
        }
    }

    /// Reconstructs and returns the CA public key.
    pub fn public_key(&self) -> Result<PublicKey, String> {
        PublicKey::from_bytes(&self.pk_bytes)
            .map_err(|_| "CA_PUBLIC_KEY_INVALID: stored public key bytes are invalid".to_string())
    }

    /// Returns the CA public key as uppercase hexadecimal.
    #[must_use]
    pub fn public_key_hex(&self) -> String {
        hex::encode_upper(&self.pk_bytes)
    }

    /// Returns true when private key material is present.
    #[must_use]
    pub fn has_private_key(&self) -> bool {
        !self.sk_bytes.is_empty()
    }

    /// Computes a stable issuer key identifier derived from the CA public key.
    ///
    /// This helper is suitable for logs, certificate metadata, registry keys,
    /// and operator-facing diagnostics.
    #[must_use]
    pub fn key_id(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(CA_KEY_ID_DOMAIN);
        hasher.update([0x00]);
        hasher.update(&self.pk_bytes);

        let digest = hasher.finalize();
        hex::encode_upper(&digest[..8])
    }

    /// Validates the issuer identifier carried by this certificate authority.
    pub fn validate_issuer(&self) -> Result<(), String> {
        validate_issuer_identifier(&self.issuer)
    }

    /// Internal secret-key reconstruction.
    fn secret_key(&self) -> Result<SecretKey, String> {
        if self.sk_bytes.is_empty() {
            return Err(
                "CA_SECRET_KEY_MISSING: certificate authority has no private key material"
                    .to_string(),
            );
        }

        SecretKey::from_bytes(&self.sk_bytes)
            .map_err(|_| "CA_SECRET_KEY_INVALID: stored secret key bytes are invalid".to_string())
    }

    /// Validates that stored key material is structurally sound and mutually consistent.
    ///
    /// Validation policy:
    /// - the public key must decode successfully,
    /// - verifier-only instances are accepted when the private key is absent,
    /// - signer-capable instances must pass a deterministic sign/verify self-test.
    fn validate_key_material_pair(&self) -> Result<(), String> {
        let pk = self.public_key()?;

        if self.sk_bytes.is_empty() {
            let _ = pk;
            return Ok(());
        }

        let sk = self.secret_key()?;
        let message = self.self_test_message();
        let signature = detached_sign(&message, &sk);

        verify_detached_signature(&signature, &message, &pk).map_err(|_| {
            "CA_KEYPAIR_MISMATCH: public and private key material are inconsistent".to_string()
        })
    }

    /// Builds the deterministic self-test message used for keypair validation.
    fn self_test_message(&self) -> Vec<u8> {
        let mut message = Vec::with_capacity(
            CA_SELF_TEST_DOMAIN.len() + 1 + self.issuer.len() + 1 + self.pk_bytes.len(),
        );

        message.extend_from_slice(CA_SELF_TEST_DOMAIN);
        message.push(0x00);
        message.extend_from_slice(self.issuer.as_bytes());
        message.push(0x00);
        message.extend_from_slice(&self.pk_bytes);

        message
    }

    /// Serializes the canonical certificate signing payload.
    ///
    /// The certificate model already exposes a canonical signing payload that
    /// excludes the detached signature field. Using that payload directly is
    /// safer and more future-proof than cloning and mutating the full certificate
    /// structure during signing operations.
    fn certificate_payload(cert: &Certificate) -> Result<Vec<u8>, String> {
        cert.signing_payload_bytes()
            .map_err(|error| format!("CERT_SERIALIZE_ERROR: {}", error))
    }

    /// Wraps a serialized certificate payload in a protocol-specific signing context.
    ///
    /// This prevents cross-domain signature reuse if the same raw payload bytes are
    /// ever used outside the certificate protocol surface.
    fn signing_message(payload: &[u8]) -> Vec<u8> {
        let mut message = Vec::with_capacity(CERTIFICATE_SIGNING_DOMAIN.len() + 1 + payload.len());

        message.extend_from_slice(CERTIFICATE_SIGNING_DOMAIN);
        message.push(0x00);
        message.extend_from_slice(payload);

        message
    }

    /// Signs a certificate using a detached Dilithium3 signature.
    ///
    /// Operational behavior:
    /// - the CA issuer string is injected into the certificate before signing;
    /// - the certificate must pass unsigned semantic validation;
    /// - the signature field is excluded from the signed payload;
    /// - the final signature is stored as an uppercase hexadecimal string;
    /// - the signed certificate must pass post-sign validation before return.
    pub fn sign_certificate(&self, mut cert: Certificate) -> Result<Certificate, String> {
        self.validate_issuer()?;
        self.validate_key_material_pair()?;

        cert.validate_unsigned()
            .map_err(|error| format!("CERT_VALIDATE_ERROR: {}", error.code()))?;

        cert.issuer = self.issuer.clone();

        let payload = Self::certificate_payload(&cert)?;
        let message = Self::signing_message(&payload);

        let sk = self.secret_key()?;
        let signature = detached_sign(&message, &sk);

        cert.signature = hex::encode_upper(signature.as_bytes());

        cert.validate_signed()
            .map_err(|error| format!("CERT_VALIDATE_ERROR: {}", error.code()))?;

        Ok(cert)
    }

    /// Verifies a certificate signature using the CA public key.
    ///
    /// Verification policy:
    /// - certificate signature must be present and decodable;
    /// - certificate issuer must match this CA instance;
    /// - the certificate must satisfy signed semantic validation;
    /// - the detached signature is verified against the domain-separated message.
    #[must_use]
    pub fn verify_certificate(&self, cert: &Certificate) -> bool {
        self.verify_certificate_detailed(cert).is_ok()
    }

    /// Verifies a certificate signature and returns detailed failure information.
    ///
    /// This method is intended for diagnostics, telemetry, and more explicit
    /// operational handling while preserving the legacy boolean interface.
    pub fn verify_certificate_detailed(&self, cert: &Certificate) -> Result<(), String> {
        self.validate_issuer()?;
        self.validate_key_material_pair()?;

        cert.validate_signed()
            .map_err(|error| format!("CERT_VERIFY_ERROR: {}", error.code()))?;

        if cert.issuer != self.issuer {
            return Err(format!(
                "CERT_VERIFY_ERROR: issuer mismatch, expected '{}' got '{}'",
                self.issuer, cert.issuer
            ));
        }

        let payload = Self::certificate_payload(cert)?;
        let message = Self::signing_message(&payload);

        let sig_bytes = hex::decode(&cert.signature)
            .map_err(|_| "CERT_VERIFY_ERROR: signature is not valid hexadecimal".to_string())?;

        let signature = DetachedSignature::from_bytes(&sig_bytes)
            .map_err(|_| "CERT_VERIFY_ERROR: detached signature bytes are invalid".to_string())?;

        let pk = self.public_key()?;

        verify_detached_signature(&signature, &message, &pk)
            .map_err(|_| "CERT_VERIFY_ERROR: detached signature verification failed".to_string())
    }

    /// Returns a clone of the certificate with its issuer normalized to this CA.
    ///
    /// This helper is useful when callers want to inspect the exact certificate
    /// content that would be signed before committing to the signing operation.
    pub fn canonicalize_certificate(&self, mut cert: Certificate) -> Certificate {
        cert.issuer = self.issuer.clone();
        cert
    }

    /// Returns the domain-separated signing payload that would be verified or signed.
    ///
    /// This function is useful for audit logs, golden tests, and protocol debugging.
    pub fn certificate_signing_payload(&self, cert: &Certificate) -> Result<Vec<u8>, String> {
        let payload = Self::certificate_payload(cert)?;
        Ok(Self::signing_message(&payload))
    }
}

/// Validates the issuer identifier accepted by the CA surface.
///
/// Validation policy:
/// - issuer must not be blank,
/// - issuer length must remain bounded,
/// - issuer may contain ASCII alphanumeric characters plus `_`, `-`, and `.`.
fn validate_issuer_identifier(value: &str) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("CA_ISSUER_INVALID: issuer must not be blank".to_string());
    }

    if trimmed.len() > MAX_ISSUER_LEN {
        return Err("CA_ISSUER_INVALID: issuer exceeds maximum supported length".to_string());
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err("CA_ISSUER_INVALID: issuer contains unsupported characters".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn key_id_is_stable_for_same_instance() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");

        let a = ca.key_id();
        let b = ca.key_id();

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn sign_and_verify_roundtrip_succeeds() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();

        let signed = ca
            .sign_certificate(cert)
            .expect("certificate signing must succeed");

        assert!(ca.verify_certificate(&signed));
    }

    #[test]
    fn verification_fails_when_signature_is_missing() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();

        assert!(!ca.verify_certificate(&cert));
    }

    #[test]
    fn verification_fails_when_certificate_is_tampered() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();

        let mut signed = ca
            .sign_certificate(cert)
            .expect("certificate signing must succeed");

        signed.actor_id = "AOXC-VAL-EU-FFFFFFFFFFFFFFFF-Z9".to_string();

        assert!(!ca.verify_certificate(&signed));
    }

    #[test]
    fn verification_fails_when_issuer_is_tampered() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = sample_certificate();

        let mut signed = ca
            .sign_certificate(cert)
            .expect("certificate signing must succeed");

        signed.issuer = "FAKE-CA".to_string();

        assert!(!ca.verify_certificate(&signed));
    }

    #[test]
    fn canonicalize_certificate_overwrites_issuer() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");

        let cert = Certificate {
            issuer: "OLD".to_string(),
            ..sample_certificate()
        };

        let canonical = ca.canonicalize_certificate(cert);
        assert_eq!(canonical.issuer, "AOXC-ROOT-CA");
    }

    #[test]
    fn verifier_only_instance_does_not_report_private_key() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let verifier = ca.to_public_verifier();

        assert!(!verifier.has_private_key());
        assert_eq!(verifier.issuer, "AOXC-ROOT-CA");
        assert_eq!(verifier.public_key_hex(), ca.public_key_hex());
    }

    #[test]
    fn from_serialized_parts_rejects_mismatched_keypair() {
        let ca_a = CertificateAuthority::new("AOXC-ROOT-CA");
        let ca_b = CertificateAuthority::new("AOXC-ROOT-CA");

        let result = CertificateAuthority::from_serialized_parts(
            "AOXC-ROOT-CA",
            ca_a.pk_bytes.clone(),
            ca_b.sk_bytes.clone(),
        );

        assert!(matches!(result, Err(error) if error.starts_with("CA_KEYPAIR_MISMATCH:")));
    }

    #[test]
    fn invalid_issuer_is_rejected_by_reconstruction() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");

        let result =
            CertificateAuthority::from_public_key_bytes("bad issuer!", ca.pk_bytes.clone());

        assert!(matches!(result, Err(error) if error.starts_with("CA_ISSUER_INVALID:")));
    }

    #[test]
    fn certificate_signing_payload_is_domain_separated() {
        let ca = CertificateAuthority::new("AOXC-ROOT-CA");
        let cert = ca.canonicalize_certificate(sample_certificate());

        let payload = ca
            .certificate_signing_payload(&cert)
            .expect("signing payload must be created");

        assert!(payload.starts_with(CERTIFICATE_SIGNING_DOMAIN));
    }
}
