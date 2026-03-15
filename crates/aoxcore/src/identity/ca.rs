use crate::identity::certificate::Certificate;

use pqcrypto_dilithium::dilithium3::{
    DetachedSignature, PublicKey, SecretKey, detached_sign, keypair, verify_detached_signature,
};

use pqcrypto_traits::sign::{DetachedSignature as _, PublicKey as _, SecretKey as _};

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

/// Canonical signing domain for AOXC certificate authority operations.
///
/// This value is prepended to the serialized certificate payload before
/// signature generation and verification. The purpose is to prevent
/// cross-domain signature confusion if the same serialized bytes appear
/// in another protocol context.
const CERTIFICATE_SIGNING_DOMAIN: &[u8] = b"AOXC/IDENTITY/CERTIFICATE/V1";

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

impl std::fmt::Debug for CertificateAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CertificateAuthority")
            .field("issuer", &self.issuer)
            .field("pk_len", &self.pk_bytes.len())
            .field("has_private_key", &(!self.sk_bytes.is_empty()))
            .finish()
    }
}

impl CertificateAuthority {
    /// Creates a new certificate authority with a freshly generated
    /// Dilithium3 keypair.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let (pk, sk) = keypair();

        Self {
            issuer: name.into(),
            sk_bytes: sk.as_bytes().to_vec(),
            pk_bytes: pk.as_bytes().to_vec(),
        }
    }

    /// Reconstructs and returns the CA public key.
    pub fn public_key(&self) -> Result<PublicKey, String> {
        PublicKey::from_bytes(&self.pk_bytes)
            .map_err(|_| "CA_PUBLIC_KEY_INVALID: stored public key bytes are invalid".to_string())
    }

    /// Returns the CA public key as uppercase hexadecimal.
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
    pub fn key_id(&self) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(b"AOXC/IDENTITY/CA/KEY_ID/V1");
        hasher.update(&self.pk_bytes);

        let digest = hasher.finalize();
        hex::encode_upper(&digest[..8])
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

    /// Serializes the certificate payload excluding the detached signature.
    ///
    /// The existing certificate model is preserved by cloning the structure and
    /// clearing the signature field prior to serialization. This keeps the CA
    /// compatible with the current certificate representation without requiring
    /// a breaking schema redesign.
    fn certificate_payload(cert: &Certificate) -> Result<Vec<u8>, String> {
        let mut tmp = cert.clone();
        tmp.signature.clear();

        serde_json::to_vec(&tmp).map_err(|e| format!("CERT_SERIALIZE_ERROR: {}", e))
    }

    /// Wraps a serialized certificate payload in a protocol-specific signing context.
    ///
    /// This prevents cross-domain signature reuse if the same raw JSON bytes are
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
    /// - the signature field is excluded from the signed payload;
    /// - the final signature is stored as an uppercase hexadecimal string.
    pub fn sign_certificate(&self, mut cert: Certificate) -> Result<Certificate, String> {
        cert.issuer = self.issuer.clone();

        let payload = Self::certificate_payload(&cert)?;
        let message = Self::signing_message(&payload);

        let sk = self.secret_key()?;
        let signature = detached_sign(&message, &sk);

        cert.signature = hex::encode_upper(signature.as_bytes());

        Ok(cert)
    }

    /// Verifies a certificate signature using the CA public key.
    ///
    /// Verification policy:
    /// - certificate signature must be present and decodable;
    /// - certificate issuer must match this CA instance;
    /// - payload is reconstructed with the signature field cleared;
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
        if cert.signature.is_empty() {
            return Err("CERT_VERIFY_ERROR: certificate signature is empty".to_string());
        }

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
}
