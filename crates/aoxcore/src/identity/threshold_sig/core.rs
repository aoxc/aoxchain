// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::BTreeSet;
use std::fmt;

/// Maximum accepted signer identifier length.
///
/// This bound is intentionally conservative and suitable for validator,
/// operator, guardian, council, or DAO signer identifiers.
pub const MAX_SIGNER_ID_LEN: usize = 128;

/// Maximum accepted threshold session identifier length.
pub const MAX_SESSION_ID_LEN: usize = 128;

/// Maximum accepted signing domain length.
pub const MAX_SIGNING_DOMAIN_LEN: usize = 128;

/// Maximum accepted serialized partial-signature length in bytes.
///
/// This upper bound is intentionally generous enough for post-quantum,
/// hybrid, or metadata-extended signature payloads while still rejecting
/// obviously malformed or unbounded input.
pub const MAX_PARTIAL_SIGNATURE_LEN: usize = 16_384;

/// Canonical payload digest size in bytes.
pub const PAYLOAD_DIGEST_LEN: usize = 32;

/// Domain separator used for payload digest derivation.
const AOXC_TSS_PAYLOAD_DIGEST_DOMAIN: &[u8] = b"AOXC/IDENTITY/TSS/PAYLOAD_DIGEST/V1";

/// Legacy threshold partial-signature envelope.
///
/// Important note:
/// This structure models only the contribution envelope. It does not itself
/// prove cryptographic validity of the contained signature bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialSignature {
    pub signer_id: String,
    pub signature: Vec<u8>,
}

impl PartialSignature {
    /// Validates the partial-signature envelope.
    pub fn validate(&self) -> Result<(), TssError> {
        validate_signer_id(&self.signer_id)?;

        if self.signature.is_empty() || self.signature.len() > MAX_PARTIAL_SIGNATURE_LEN {
            return Err(TssError::InvalidPartialSignature);
        }

        Ok(())
    }
}

/// Minimal legacy threshold policy.
///
/// Compatibility note:
/// this structure is intentionally preserved to avoid breaking existing call sites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdPolicy {
    pub min_signers: usize,
}

impl ThresholdPolicy {
    #[must_use]
    pub const fn new(min_signers: usize) -> Self {
        Self { min_signers }
    }

    /// Validates the threshold policy as a self-consistent approval contract.
    pub fn validate(&self) -> Result<(), TssError> {
        if self.min_signers == 0 {
            return Err(TssError::InvalidPolicy);
        }

        Ok(())
    }
}

/// Session-bound partial signature.
///
/// This structure binds a signer contribution to:
/// - a specific threshold session,
/// - a specific round,
/// - a specific canonical payload digest.
///
/// This reduces replay risk and prevents cross-session contribution reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionBoundPartialSignature {
    pub signer_id: String,
    pub signature: Vec<u8>,
    pub session_id: String,
    pub round: u64,
    pub payload_digest: [u8; PAYLOAD_DIGEST_LEN],
}

impl SessionBoundPartialSignature {
    /// Validates the bound partial-signature envelope.
    pub fn validate(&self) -> Result<(), TssError> {
        validate_signer_id(&self.signer_id)?;
        validate_session_id(&self.session_id)?;

        if self.round == 0 {
            return Err(TssError::InvalidRound);
        }

        if self.signature.is_empty() || self.signature.len() > MAX_PARTIAL_SIGNATURE_LEN {
            return Err(TssError::InvalidPartialSignature);
        }

        if self.payload_digest.iter().all(|byte| *byte == 0) {
            return Err(TssError::InvalidPayloadDigest);
        }

        Ok(())
    }
}

/// Canonical threshold session context.
///
/// This structure defines the exact approval context that partial signatures
/// are expected to bind to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdSessionContext {
    pub session_id: String,
    pub signing_domain: String,
    pub round: u64,
    pub payload_digest: [u8; PAYLOAD_DIGEST_LEN],
    pub authorized_signers: BTreeSet<String>,
    pub issued_at: u64,
    pub expires_at: u64,
}

impl ThresholdSessionContext {
    /// Validates the full threshold session context.
    pub fn validate(&self) -> Result<(), TssError> {
        validate_session_id(&self.session_id)?;
        validate_signing_domain(&self.signing_domain)?;

        if self.round == 0 {
            return Err(TssError::InvalidRound);
        }

        if self.payload_digest.iter().all(|byte| *byte == 0) {
            return Err(TssError::InvalidPayloadDigest);
        }

        if self.issued_at == 0 || self.expires_at == 0 || self.expires_at <= self.issued_at {
            return Err(TssError::InvalidSessionWindow);
        }

        if self.authorized_signers.is_empty() {
            return Err(TssError::InvalidAuthorizedSignerSet);
        }

        for signer_id in &self.authorized_signers {
            validate_signer_id(signer_id)?;
        }

        Ok(())
    }

    /// Returns whether the session is valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_valid_at(&self, now: u64) -> bool {
        now >= self.issued_at && now < self.expires_at
    }

    /// Returns whether the session is expired at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_expired_at(&self, now: u64) -> bool {
        now >= self.expires_at
    }

    /// Returns whether the session is not yet valid at the supplied UNIX timestamp.
    #[must_use]
    pub fn is_not_yet_valid_at(&self, now: u64) -> bool {
        now < self.issued_at
    }
}

/// Extended threshold session policy.
///
/// Security model:
/// - preserves the legacy minimum signer count,
/// - optionally enforces signer membership against an allowlist,
/// - optionally enforces exact round binding,
/// - optionally rejects oversubscribed signer sets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdSessionPolicy {
    pub threshold: ThresholdPolicy,
    pub require_authorized_signers: bool,
    pub require_matching_round: bool,
    pub reject_oversubscription: bool,
}

impl ThresholdSessionPolicy {
    #[must_use]
    pub const fn new(min_signers: usize) -> Self {
        Self {
            threshold: ThresholdPolicy::new(min_signers),
            require_authorized_signers: true,
            require_matching_round: true,
            reject_oversubscription: false,
        }
    }

    /// Returns a strict default policy suitable for protected operational flows.
    #[must_use]
    pub const fn strict_default(min_signers: usize) -> Self {
        Self {
            threshold: ThresholdPolicy::new(min_signers),
            require_authorized_signers: true,
            require_matching_round: true,
            reject_oversubscription: true,
        }
    }

    /// Validates the session policy.
    pub fn validate(&self) -> Result<(), TssError> {
        self.threshold.validate()
    }
}

/// Structured threshold-verification result.
///
/// This is useful for audit logs, telemetry, and higher-level approval logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThresholdVerificationReport {
    pub accepted_signers: Vec<String>,
    pub unique_signer_count: usize,
    pub threshold_required: usize,
    pub session_id: String,
    pub round: u64,
    pub payload_digest: [u8; PAYLOAD_DIGEST_LEN],
}

/// Canonical threshold-signature validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TssError {
    InvalidPolicy,
    InvalidSessionPolicy,
    InvalidSignerId,
    InvalidSessionId,
    InvalidSigningDomain,
    InvalidRound,
    InvalidPayloadDigest,
    InvalidAuthorizedSignerSet,
    InvalidSessionWindow,
    InvalidPartialSignature,
    DuplicateSigner,
    UnauthorizedSigner,
    SessionMismatch,
    RoundMismatch,
    PayloadMismatch,
    SessionNotYetValid,
    SessionExpired,
    QuorumNotReached,
    TooManySigners,
    SignatureBackendRejected,
}

impl TssError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidPolicy => "TSS_INVALID_POLICY",
            Self::InvalidSessionPolicy => "TSS_INVALID_SESSION_POLICY",
            Self::InvalidSignerId => "TSS_INVALID_SIGNER_ID",
            Self::InvalidSessionId => "TSS_INVALID_SESSION_ID",
            Self::InvalidSigningDomain => "TSS_INVALID_SIGNING_DOMAIN",
            Self::InvalidRound => "TSS_INVALID_ROUND",
            Self::InvalidPayloadDigest => "TSS_INVALID_PAYLOAD_DIGEST",
            Self::InvalidAuthorizedSignerSet => "TSS_INVALID_AUTHORIZED_SIGNER_SET",
            Self::InvalidSessionWindow => "TSS_INVALID_SESSION_WINDOW",
            Self::InvalidPartialSignature => "TSS_INVALID_PARTIAL_SIGNATURE",
            Self::DuplicateSigner => "TSS_DUPLICATE_SIGNER",
            Self::UnauthorizedSigner => "TSS_UNAUTHORIZED_SIGNER",
            Self::SessionMismatch => "TSS_SESSION_MISMATCH",
            Self::RoundMismatch => "TSS_ROUND_MISMATCH",
            Self::PayloadMismatch => "TSS_PAYLOAD_MISMATCH",
            Self::SessionNotYetValid => "TSS_SESSION_NOT_YET_VALID",
            Self::SessionExpired => "TSS_SESSION_EXPIRED",
            Self::QuorumNotReached => "TSS_QUORUM_NOT_REACHED",
            Self::TooManySigners => "TSS_TOO_MANY_SIGNERS",
            Self::SignatureBackendRejected => "TSS_SIGNATURE_BACKEND_REJECTED",
        }
    }
}

impl fmt::Display for TssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl std::error::Error for TssError {}

/// Backend verification trait for cryptographic partial-signature validation.
///
/// This module intentionally separates:
/// - structural/session/policy verification,
/// - actual cryptographic verification.
///
/// Concrete threshold-signature backends should implement this trait.
pub trait PartialSignatureVerifier {
    /// Verifies that a session-bound partial signature is cryptographically
    /// valid for the supplied threshold session context.
    fn verify_partial(
        &self,
        session: &ThresholdSessionContext,
        partial: &SessionBoundPartialSignature,
    ) -> Result<(), TssError>;
}

/// Verifies legacy threshold-signature envelope requirements.
///
/// Validation scope:
/// - policy must be valid,
/// - every partial signature must be structurally valid,
/// - every signer identifier must be canonical,
/// - duplicate signer participation is rejected,
/// - quorum is satisfied only by distinct valid signers.
///
/// Important note:
/// This function validates threshold-envelope integrity only. It does not
/// perform cryptographic verification of the partial signatures themselves.
pub fn verify_threshold_signatures(
    policy: &ThresholdPolicy,
    partials: &[PartialSignature],
) -> Result<(), String> {
    verify_threshold_signatures_detailed(policy, partials).map_err(|error| error.code().to_string())
}

/// Detailed legacy threshold-signature verification.
///
/// New internal call paths should prefer this function where structured error
/// handling is more useful than symbolic string codes.
pub fn verify_threshold_signatures_detailed(
    policy: &ThresholdPolicy,
    partials: &[PartialSignature],
) -> Result<(), TssError> {
    policy.validate()?;

    let mut unique_signers = BTreeSet::new();

    for partial in partials {
        partial.validate()?;

        if !unique_signers.insert(partial.signer_id.clone()) {
            return Err(TssError::DuplicateSigner);
        }
    }

    if unique_signers.len() < policy.min_signers {
        return Err(TssError::QuorumNotReached);
    }

    Ok(())
}

/// Verifies a session-bound threshold-signature set without a cryptographic backend.
///
/// This function enforces:
/// - policy validity,
/// - session validity,
/// - exact session and payload binding,
/// - unique signer participation,
/// - optional signer allowlist membership,
/// - quorum satisfaction.
///
/// Important note:
/// This function does not perform cryptographic verification of signature bytes.
/// For a full production path, prefer
/// `verify_threshold_session_signatures_with_verifier(...)`.
pub fn verify_threshold_session_signatures(
    policy: &ThresholdSessionPolicy,
    session: &ThresholdSessionContext,
    partials: &[SessionBoundPartialSignature],
    now: u64,
) -> Result<ThresholdVerificationReport, TssError> {
    let verifier = NoopVerifier;
    verify_threshold_session_signatures_internal(policy, session, partials, now, Some(&verifier))
}

/// Verifies a session-bound threshold-signature set with a cryptographic backend.
///
/// This is the preferred production-grade entry point.
pub fn verify_threshold_session_signatures_with_verifier<V: PartialSignatureVerifier>(
    policy: &ThresholdSessionPolicy,
    session: &ThresholdSessionContext,
    partials: &[SessionBoundPartialSignature],
    now: u64,
    verifier: &V,
) -> Result<ThresholdVerificationReport, TssError> {
    verify_threshold_session_signatures_internal(policy, session, partials, now, Some(verifier))
}

/// Computes the canonical AOXC threshold-signing payload digest.
///
/// This digest should be the exact value carried inside `ThresholdSessionContext`
/// and `SessionBoundPartialSignature`.
#[must_use]
pub fn compute_payload_digest(payload: &[u8]) -> [u8; PAYLOAD_DIGEST_LEN] {
    let mut hasher = Sha3_256::new();
    hasher.update(AOXC_TSS_PAYLOAD_DIGEST_DOMAIN);
    hasher.update([0x00]);
    hasher.update(payload);

    let digest = hasher.finalize();

    let mut out = [0u8; PAYLOAD_DIGEST_LEN];
    out.copy_from_slice(&digest[..PAYLOAD_DIGEST_LEN]);
    out
}

/// Internal shared verification flow.
fn verify_threshold_session_signatures_internal<V: PartialSignatureVerifier>(
    policy: &ThresholdSessionPolicy,
    session: &ThresholdSessionContext,
    partials: &[SessionBoundPartialSignature],
    now: u64,
    verifier: Option<&V>,
) -> Result<ThresholdVerificationReport, TssError> {
    policy
        .validate()
        .map_err(|_| TssError::InvalidSessionPolicy)?;
    session.validate()?;

    if session.is_not_yet_valid_at(now) {
        return Err(TssError::SessionNotYetValid);
    }

    if session.is_expired_at(now) {
        return Err(TssError::SessionExpired);
    }

    if policy.reject_oversubscription && partials.len() > session.authorized_signers.len() {
        return Err(TssError::TooManySigners);
    }

    let mut accepted_signers = BTreeSet::new();

    for partial in partials {
        partial.validate()?;

        if partial.session_id != session.session_id {
            return Err(TssError::SessionMismatch);
        }

        if policy.require_matching_round && partial.round != session.round {
            return Err(TssError::RoundMismatch);
        }

        if partial.payload_digest != session.payload_digest {
            return Err(TssError::PayloadMismatch);
        }

        if policy.require_authorized_signers
            && !session.authorized_signers.contains(&partial.signer_id)
        {
            return Err(TssError::UnauthorizedSigner);
        }

        if !accepted_signers.insert(partial.signer_id.clone()) {
            return Err(TssError::DuplicateSigner);
        }

        if let Some(backend) = verifier {
            backend
                .verify_partial(session, partial)
                .map_err(|_| TssError::SignatureBackendRejected)?;
        }
    }

    let unique_signer_count = accepted_signers.len();

    if unique_signer_count < policy.threshold.min_signers {
        return Err(TssError::QuorumNotReached);
    }

    Ok(ThresholdVerificationReport {
        accepted_signers: accepted_signers.into_iter().collect(),
        unique_signer_count,
        threshold_required: policy.threshold.min_signers,
        session_id: session.session_id.clone(),
        round: session.round,
        payload_digest: session.payload_digest,
    })
}

/// Validates a signer identifier.
///
/// Policy:
/// - must not be blank,
/// - surrounding whitespace is rejected rather than normalized,
/// - length must remain bounded,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted.
fn validate_signer_id(signer_id: &str) -> Result<(), TssError> {
    validate_identifier(signer_id, MAX_SIGNER_ID_LEN).map_err(|_| TssError::InvalidSignerId)
}

/// Validates a threshold session identifier.
fn validate_session_id(session_id: &str) -> Result<(), TssError> {
    validate_identifier(session_id, MAX_SESSION_ID_LEN).map_err(|_| TssError::InvalidSessionId)
}

/// Validates a signing domain identifier.
fn validate_signing_domain(signing_domain: &str) -> Result<(), TssError> {
    validate_identifier(signing_domain, MAX_SIGNING_DOMAIN_LEN)
        .map_err(|_| TssError::InvalidSigningDomain)
}

/// Shared identifier validator.
fn validate_identifier(value: &str, max_len: usize) -> Result<(), ()> {
    if value.is_empty() || value.trim().is_empty() {
        return Err(());
    }

    if value != value.trim() {
        return Err(());
    }

    if value.len() > max_len {
        return Err(());
    }

    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(());
    }

    Ok(())
}

/// No-op verifier used by the envelope-only validation path.
struct NoopVerifier;

impl PartialSignatureVerifier for NoopVerifier {
    fn verify_partial(
        &self,
        _session: &ThresholdSessionContext,
        _partial: &SessionBoundPartialSignature,
    ) -> Result<(), TssError> {
        Ok(())
    }
}
