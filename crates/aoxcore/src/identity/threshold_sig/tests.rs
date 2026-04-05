use super::*;
use std::collections::BTreeSet;

struct AcceptAllVerifier;

impl PartialSignatureVerifier for AcceptAllVerifier {
    fn verify_partial(
        &self,
        _session: &ThresholdSessionContext,
        _partial: &SessionBoundPartialSignature,
    ) -> Result<(), TssError> {
        Ok(())
    }
}

struct RejectAllVerifier;

impl PartialSignatureVerifier for RejectAllVerifier {
    fn verify_partial(
        &self,
        _session: &ThresholdSessionContext,
        _partial: &SessionBoundPartialSignature,
    ) -> Result<(), TssError> {
        Err(TssError::SignatureBackendRejected)
    }
}

fn partial(signer_id: &str, signature: Vec<u8>) -> PartialSignature {
    PartialSignature {
        signer_id: signer_id.to_string(),
        signature,
    }
}

fn bound_partial(
    signer_id: &str,
    session_id: &str,
    round: u64,
    payload_digest: [u8; PAYLOAD_DIGEST_LEN],
) -> SessionBoundPartialSignature {
    SessionBoundPartialSignature {
        signer_id: signer_id.to_string(),
        signature: vec![1, 2, 3, 4],
        session_id: session_id.to_string(),
        round,
        payload_digest,
    }
}

fn session() -> ThresholdSessionContext {
    let payload_digest = compute_payload_digest(b"AOXC protected payload");

    ThresholdSessionContext {
        session_id: "session-001".to_string(),
        signing_domain: "AOXC.TSS.OPS".to_string(),
        round: 1,
        payload_digest,
        authorized_signers: BTreeSet::from([
            "validator-1".to_string(),
            "validator-2".to_string(),
            "validator-3".to_string(),
        ]),
        issued_at: 100,
        expires_at: 200,
    }
}

#[test]
fn threshold_reached() {
    let policy = ThresholdPolicy::new(2);
    let partials = vec![
        partial("validator-1", vec![1, 2, 3]),
        partial("validator-2", vec![4, 5, 6]),
    ];

    assert!(verify_threshold_signatures(&policy, &partials).is_ok());
}

#[test]
fn invalid_policy_is_rejected() {
    let policy = ThresholdPolicy::new(0);
    let partials = vec![partial("validator-1", vec![1, 2, 3])];

    let error = verify_threshold_signatures(&policy, &partials).unwrap_err();
    assert_eq!(error, "TSS_INVALID_POLICY");
}

#[test]
fn duplicate_signer_is_rejected() {
    let policy = ThresholdPolicy::new(2);
    let partials = vec![
        partial("validator-1", vec![1, 2, 3]),
        partial("validator-1", vec![4, 5, 6]),
    ];

    let error = verify_threshold_signatures(&policy, &partials).unwrap_err();
    assert_eq!(error, "TSS_DUPLICATE_SIGNER");
}

#[test]
fn session_bound_threshold_verification_succeeds() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, session.payload_digest),
    ];

    let report = verify_threshold_session_signatures(&policy, &session, &partials, 150).unwrap();

    assert_eq!(report.threshold_required, 2);
    assert_eq!(report.session_id, "session-001");
    assert_eq!(report.round, 1);
    assert_eq!(report.accepted_signers.len(), 2);
    assert_eq!(report.unique_signer_count, 2);
}

#[test]
fn unauthorized_signer_is_rejected() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("attacker-1", "session-001", 1, session.payload_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 150).unwrap_err();

    assert_eq!(error, TssError::UnauthorizedSigner);
}

#[test]
fn payload_mismatch_is_rejected() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let wrong_digest = compute_payload_digest(b"tampered payload");

    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, wrong_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 150).unwrap_err();

    assert_eq!(error, TssError::PayloadMismatch);
}

#[test]
fn round_mismatch_is_rejected() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 2, session.payload_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 150).unwrap_err();

    assert_eq!(error, TssError::RoundMismatch);
}

#[test]
fn session_mismatch_is_rejected() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-XYZ", 1, session.payload_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 150).unwrap_err();

    assert_eq!(error, TssError::SessionMismatch);
}

#[test]
fn session_expiry_is_enforced() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, session.payload_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 250).unwrap_err();

    assert_eq!(error, TssError::SessionExpired);
}

#[test]
fn session_not_yet_valid_is_enforced() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, session.payload_digest),
    ];

    let error = verify_threshold_session_signatures(&policy, &session, &partials, 50).unwrap_err();

    assert_eq!(error, TssError::SessionNotYetValid);
}

#[test]
fn verifier_hook_can_accept_backend_validation() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, session.payload_digest),
    ];

    let verifier = AcceptAllVerifier;
    let result = verify_threshold_session_signatures_with_verifier(
        &policy, &session, &partials, 150, &verifier,
    );

    assert!(result.is_ok());
}

#[test]
fn verifier_hook_can_reject_backend_validation() {
    let policy = ThresholdSessionPolicy::strict_default(2);
    let session = session();
    let partials = vec![
        bound_partial("validator-1", "session-001", 1, session.payload_digest),
        bound_partial("validator-2", "session-001", 1, session.payload_digest),
    ];

    let verifier = RejectAllVerifier;
    let error = verify_threshold_session_signatures_with_verifier(
        &policy, &session, &partials, 150, &verifier,
    )
    .unwrap_err();

    assert_eq!(error, TssError::SignatureBackendRejected);
}

#[test]
fn payload_digest_is_stable() {
    let a = compute_payload_digest(b"AOXC payload");
    let b = compute_payload_digest(b"AOXC payload");
    let c = compute_payload_digest(b"AOXC other payload");

    assert_eq!(a, b);
    assert_ne!(a, c);
}
