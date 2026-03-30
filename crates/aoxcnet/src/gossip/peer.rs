// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::{ExternalDomainKind, NetworkConfig, SecurityMode};
use crate::error::NetworkError;

/// Certificate metadata used for secure peer admission.
///
/// The certificate model is intentionally minimal at this stage, but it still
/// carries the core fields required for deterministic fingerprinting, temporal
/// validity checks, and domain-level attestation enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeCertificate {
    /// Canonical certificate subject bound to the peer identity.
    pub subject: String,

    /// Canonical issuing authority for the certificate.
    pub issuer: String,

    /// Lower bound of certificate validity in UNIX seconds.
    pub valid_from_unix: u64,

    /// Upper bound of certificate validity in UNIX seconds.
    pub valid_until_unix: u64,

    /// Certificate serial identifier as issued by the authority.
    pub serial: String,

    /// Domain-level attestation hash used when inter-domain trust enforcement
    /// is enabled by runtime policy.
    pub domain_attestation_hash: String,
}

impl NodeCertificate {
    /// Returns a deterministic certificate fingerprint.
    ///
    /// The fingerprint is derived from identity-bearing certificate fields so
    /// that peer-session establishment can bind to a stable admission artifact.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.subject.as_bytes());
        hasher.update(self.issuer.as_bytes());
        hasher.update(self.serial.as_bytes());
        hasher.update(self.valid_from_unix.to_le_bytes());
        hasher.update(self.valid_until_unix.to_le_bytes());
        hasher.update(self.domain_attestation_hash.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Returns `true` when the certificate is valid at the supplied UNIX time.
    #[must_use]
    pub fn is_valid_at(&self, unix_ts: u64) -> bool {
        unix_ts >= self.valid_from_unix && unix_ts <= self.valid_until_unix
    }

    /// Performs static certificate sanity validation independent of wall-clock
    /// time. These checks catch structurally weak certificate records before
    /// temporal validation is even considered.
    pub fn validate_structure(&self) -> Result<(), NetworkError> {
        if self.subject.trim().is_empty() {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate subject must not be empty".to_string(),
            ));
        }

        if self.issuer.trim().is_empty() {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate issuer must not be empty".to_string(),
            ));
        }

        if self.serial.trim().is_empty() {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate serial must not be empty".to_string(),
            ));
        }

        if self.valid_until_unix < self.valid_from_unix {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate validity window is inconsistent".to_string(),
            ));
        }

        Ok(())
    }
}

/// Defines the canonical operational role of a peer inside the AOXC network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerRole {
    Validator,
    Relay,
    Observer,
    Bridge,
}

impl PeerRole {
    /// Returns `true` when the role is considered part of the elevated scrutiny
    /// lane under runtime inspection policy.
    #[must_use]
    pub fn requires_inspection_lane(self) -> bool {
        matches!(self, Self::Relay | Self::Bridge)
    }

    /// Returns `true` when the role is considered consensus-sensitive.
    #[must_use]
    pub fn is_consensus_critical(self) -> bool {
        matches!(self, Self::Validator)
    }
}

/// Representation of a network peer.
///
/// The peer record intentionally combines transport identity, domain
/// classification, admission metadata, and certificate binding so that the
/// runtime can make deterministic policy decisions at registration time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peer {
    /// Canonical peer identifier used by the runtime.
    pub id: String,

    /// Operator-provided transport address advertised for peer reachability.
    pub address: String,

    /// Canonical peer chain identifier declared by the remote party.
    pub chain_id: String,

    /// Declared execution-domain family of the remote peer.
    pub domain: ExternalDomainKind,

    /// Canonical operational role of the peer.
    pub role: PeerRole,

    /// KYC assurance tier bound to this peer for regulated flows.
    pub kyc_tier: u8,

    /// Indicates whether the peer is enrolled in the AI inspection lane when
    /// such enforcement is required by runtime policy.
    pub ai_inspection_ready: bool,

    /// Certificate metadata presented by the peer.
    pub certificate: NodeCertificate,

    /// Deterministic fingerprint derived from the certificate.
    pub cert_fingerprint: String,
}

impl Peer {
    /// Creates a new peer record and deterministically derives its certificate
    /// fingerprint.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        address: impl Into<String>,
        chain_id: impl Into<String>,
        domain: ExternalDomainKind,
        role: PeerRole,
        kyc_tier: u8,
        ai_inspection_ready: bool,
        certificate: NodeCertificate,
    ) -> Self {
        let cert_fingerprint = certificate.fingerprint();

        Self {
            id: id.into(),
            address: address.into(),
            chain_id: chain_id.into(),
            domain,
            role,
            kyc_tier,
            ai_inspection_ready,
            certificate,
            cert_fingerprint,
        }
    }

    /// Returns the current UNIX timestamp in seconds.
    #[must_use]
    fn unix_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }

    /// Returns `true` when the declared peer identity is minimally well-formed.
    #[must_use]
    pub fn has_valid_identity_shape(&self) -> bool {
        !self.id.trim().is_empty()
            && !self.address.trim().is_empty()
            && !self.chain_id.trim().is_empty()
            && !self.cert_fingerprint.trim().is_empty()
    }

    /// Returns `true` when the peer belongs to the canonical local chain.
    #[must_use]
    pub fn is_native_to(&self, config: &NetworkConfig) -> bool {
        self.chain_id == config.interop.canonical_chain_id()
    }

    /// Enforces the full mainnet-grade peer admission policy.
    ///
    /// This method validates identity shape, certificate structure, temporal
    /// certificate validity, domain interoperability policy, attestation
    /// requirements, inspection-lane readiness, and role-specific admission
    /// constraints.
    pub fn validate_certificate(&self, config: &NetworkConfig) -> Result<(), NetworkError> {
        self.validate_certificate_structure_and_identity()?;

        if matches!(config.security_mode, SecurityMode::Insecure) {
            return self.validate_policy_only(config);
        }

        let now = Self::unix_now();
        if !self.certificate.is_valid_at(now) {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate expired or not yet valid".to_string(),
            ));
        }

        self.validate_policy_only(config)
    }

    /// Compatibility helper for call sites that only pass the security mode.
    ///
    /// This method intentionally performs only time-validity enforcement and is
    /// suitable for lightweight callers. Full admission logic should prefer
    /// `validate_certificate(&NetworkConfig)`.
    pub fn validate_certificate_for_mode(
        &self,
        security_mode: SecurityMode,
    ) -> Result<(), NetworkError> {
        self.validate_certificate_structure_and_identity()?;

        if matches!(security_mode, SecurityMode::Insecure) {
            return Ok(());
        }

        let now = Self::unix_now();
        if self.certificate.is_valid_at(now) {
            return Ok(());
        }

        Err(NetworkError::CertificateValidationFailed(
            "certificate expired or not yet valid".to_string(),
        ))
    }

    /// Validates identity-bearing peer fields and certificate structural
    /// integrity.
    fn validate_certificate_structure_and_identity(&self) -> Result<(), NetworkError> {
        if self.id.trim().is_empty() {
            return Err(NetworkError::PeerAdmissionDenied(
                "peer id must not be empty".to_string(),
            ));
        }

        if self.address.trim().is_empty() {
            return Err(NetworkError::PeerAdmissionDenied(
                "peer address must not be empty".to_string(),
            ));
        }

        if self.chain_id.trim().is_empty() {
            return Err(NetworkError::PeerAdmissionDenied(
                "peer chain id must not be empty".to_string(),
            ));
        }

        if self.cert_fingerprint != self.certificate.fingerprint() {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate fingerprint does not match certificate body".to_string(),
            ));
        }

        self.certificate.validate_structure()
    }

    /// Validates runtime policy constraints that are independent of raw
    /// certificate time-validity.
    fn validate_policy_only(&self, config: &NetworkConfig) -> Result<(), NetworkError> {
        if config.requires_mutual_auth() && self.certificate.subject != self.id {
            return Err(NetworkError::CertificateValidationFailed(
                "certificate subject must match peer id under mutual authentication".to_string(),
            ));
        }

        if config.requires_mutual_auth() && self.certificate.issuer == self.certificate.subject {
            return Err(NetworkError::CertificateValidationFailed(
                "self-issued peer certificates are not allowed under mutual authentication"
                    .to_string(),
            ));
        }

        if !config.interop.allow_external_domains && self.domain != ExternalDomainKind::Native {
            return Err(NetworkError::PeerAdmissionDenied(
                "external domain admission is disabled by interop policy".to_string(),
            ));
        }

        if !config.interop.allowed_domains.contains(&self.domain) {
            return Err(NetworkError::PeerAdmissionDenied(
                "peer domain is not present in the allowed domain set".to_string(),
            ));
        }

        if self.domain == ExternalDomainKind::Native && !self.is_native_to(config) {
            return Err(NetworkError::PeerAdmissionDenied(
                "native peer chain identifier does not match canonical local chain identity"
                    .to_string(),
            ));
        }

        if config.interop.require_domain_attestation
            && self.certificate.domain_attestation_hash.trim().is_empty()
        {
            return Err(NetworkError::CertificateValidationFailed(
                "domain attestation hash missing".to_string(),
            ));
        }

        if matches!(self.role, PeerRole::Bridge)
            && self.kyc_tier < config.inspection.minimum_bridge_kyc_tier
        {
            return Err(NetworkError::PeerAdmissionDenied(
                "bridge peer does not satisfy minimum KYC tier".to_string(),
            ));
        }

        if config.inspection.enable_ai_inspection_lane
            && self.role.requires_inspection_lane()
            && !self.ai_inspection_ready
        {
            return Err(NetworkError::PeerAdmissionDenied(
                "peer required by inspection policy is not AI-inspection ready".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_certificate() -> NodeCertificate {
        NodeCertificate {
            subject: "node-1".to_string(),
            issuer: "AOXC-ROOT".to_string(),
            valid_from_unix: 1,
            valid_until_unix: u64::MAX,
            serial: "serial-1".to_string(),
            domain_attestation_hash: "attestation-hash-1".to_string(),
        }
    }

    fn base_peer() -> Peer {
        Peer::new(
            "node-1",
            "10.0.0.1:2727",
            "AOXC-MAINNET",
            ExternalDomainKind::Native,
            PeerRole::Validator,
            3,
            true,
            base_certificate(),
        )
    }

    #[test]
    fn mutual_auth_rejects_subject_peer_id_mismatch() {
        let mut peer = base_peer();
        peer.certificate.subject = "other-node".to_string();
        peer.cert_fingerprint = peer.certificate.fingerprint();

        let err = peer
            .validate_certificate(&NetworkConfig::default())
            .expect_err("subject mismatch must be rejected");

        assert!(matches!(err, NetworkError::CertificateValidationFailed(_)));
    }

    #[test]
    fn mutual_auth_rejects_self_issued_certificate() {
        let mut peer = base_peer();
        peer.certificate.issuer = peer.certificate.subject.clone();
        peer.cert_fingerprint = peer.certificate.fingerprint();

        let err = peer
            .validate_certificate(&NetworkConfig::default())
            .expect_err("self-issued cert must be rejected");

        assert!(matches!(err, NetworkError::CertificateValidationFailed(_)));
    }

    #[test]
    fn peer_validation_rejects_fingerprint_drift() {
        let mut peer = base_peer();
        peer.cert_fingerprint = "tampered".to_string();

        let err = peer
            .validate_certificate(&NetworkConfig::default())
            .expect_err("fingerprint drift must be rejected");

        assert!(matches!(err, NetworkError::CertificateValidationFailed(_)));
    }
}
