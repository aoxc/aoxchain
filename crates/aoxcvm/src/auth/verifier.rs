//! Registry-backed auth envelope verification helpers.

use crate::{
    auth::{
        constitution::CryptographicConstitution,
        domains::AuthDomain,
        envelope::{AuthEnvelope, AuthEnvelopeLimits},
        qrkf::{AuthorizationLane, EpochKeyBundle, LanePolicy},
        registry::{AuthProfileId, AuthProfileRegistry},
    },
    errors::AoxcvmError,
    errors::AoxcvmResult,
};

/// Deterministic verification result context.
///
/// This structure is intentionally minimal and immutable in meaning:
/// - `profile_id` identifies the canonical registry profile used for verification
/// - `profile_version` records the exact resolved profile version
/// - `signer_count` captures the number of envelope signers accepted for evaluation
///
/// The context is returned only after all requested verification stages succeed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedAuthContext {
    pub profile_id: AuthProfileId,
    pub profile_version: u16,
    pub signer_count: usize,
}

/// Verifies an auth envelope against a canonical registry profile.
///
/// Security model:
/// 1. Resolve the requested profile version, or the latest canonical version
/// 2. Delegate envelope validation to the resolved registry record
/// 3. Return a deterministic verification context only on success
///
/// This function does not apply constitution-specific or QRKF-specific controls.
/// Those controls are layered explicitly by higher-order verification entry points.
pub fn verify_envelope(
    registry: &AuthProfileRegistry,
    profile_id: AuthProfileId,
    profile_version: Option<u16>,
    envelope: &AuthEnvelope,
    limits: AuthEnvelopeLimits,
) -> AoxcvmResult<VerifiedAuthContext> {
    let (resolved_version, record) = match profile_version {
        Some(version) => (version, registry.get_version(profile_id, version)?),
        None => registry.get_latest(profile_id)?,
    };

    record.validate_envelope(envelope, limits)?;

    Ok(VerifiedAuthContext {
        profile_id,
        profile_version: resolved_version,
        signer_count: envelope.signers.len(),
    })
}

/// Additional QRKF constraints that can be enforced on top of envelope checks.
///
/// This layer is intentionally separate from baseline envelope verification
/// so that callers can compose policy stacks explicitly and deterministically.
#[derive(Debug, Clone)]
pub struct QrkfVerification<'a> {
    pub lane_policy: LanePolicy,
    pub lane_approvals: &'a [AuthorizationLane],
    pub current_bundle: &'a EpochKeyBundle,
    pub predecessor_bundle: Option<&'a EpochKeyBundle>,
}

/// Verifies an envelope and enforces QRKF lane and continuity constraints.
///
/// Failure ordering is deterministic:
/// 1. Baseline registry-backed envelope verification
/// 2. Lane policy satisfaction
/// 3. Epoch continuity validation
///
/// This ordering is security-relevant because malformed or non-compliant envelopes
/// must fail before higher-level policy assertions are evaluated.
pub fn verify_envelope_with_qrkf(
    registry: &AuthProfileRegistry,
    profile_id: AuthProfileId,
    profile_version: Option<u16>,
    envelope: &AuthEnvelope,
    limits: AuthEnvelopeLimits,
    qrkf: QrkfVerification<'_>,
) -> AoxcvmResult<VerifiedAuthContext> {
    let verified = verify_envelope(registry, profile_id, profile_version, envelope, limits)?;

    if !qrkf.lane_policy.is_satisfied(qrkf.lane_approvals) {
        return Err(AoxcvmError::PolicyViolation(
            "qrkf failed: authorization lane policy not satisfied",
        ));
    }

    if !qrkf
        .current_bundle
        .continuity_is_valid(qrkf.predecessor_bundle)
    {
        return Err(AoxcvmError::PolicyViolation(
            "qrkf failed: epoch continuity chain is invalid",
        ));
    }

    Ok(verified)
}

/// Verifies registry profile checks and constitution checks in one pass.
///
/// Domain handling is explicit and fail-closed:
/// - Constitutional recovery domains are evaluated under the recovery rule set
/// - Other recognized domains are evaluated under the operational rule set
/// - Unrecognized domains are rejected
pub fn verify_envelope_under_constitution(
    registry: &AuthProfileRegistry,
    profile_id: AuthProfileId,
    profile_version: Option<u16>,
    envelope: &AuthEnvelope,
    limits: AuthEnvelopeLimits,
    constitution: &CryptographicConstitution,
) -> AoxcvmResult<VerifiedAuthContext> {
    let verified = verify_envelope(registry, profile_id, profile_version, envelope, limits)?;

    match AuthDomain::parse(envelope.domain.as_str()) {
        Some(AuthDomain::ConstitutionalRecovery) => {
            constitution.validate_constitutional_recovery_envelope(envelope)?
        }
        Some(_) => constitution.validate_operational_envelope(envelope)?,
        None => {
            return Err(AoxcvmError::InvalidSignatureMetadata(
                "domain must be recognized for constitution verification",
            ));
        }
    }

    Ok(verified)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry},
        policy::QuorumPolicy,
        qrkf::{AuthorizationLane, CryptoProfileId, EpochKeyBundle, KeyRealm, LanePolicy},
        registry::{AuthProfileId, AuthProfileRecord, AuthProfileRegistry},
        scheme::{AuthProfile, SignatureAlgorithm},
        signer::SignerClass,
        threshold::ThresholdPolicy,
        verifier::{
            QrkfVerification, verify_envelope, verify_envelope_under_constitution,
            verify_envelope_with_qrkf,
        },
    };

    /// Returns a deterministic dummy signature payload whose length is compatible
    /// with the selected signature algorithm for test-only metadata validation.
    ///
    /// Important:
    /// These sizes must remain aligned with the production-side signature metadata
    /// validator. The values below are chosen to satisfy realistic post-quantum
    /// algorithm length expectations rather than using undersized placeholders.
    fn valid_test_signature(algorithm: SignatureAlgorithm, fill: u8) -> Vec<u8> {
        let len = match algorithm {
            SignatureAlgorithm::MlDsa65 => 3309,
            SignatureAlgorithm::MlDsa87 => 4627,
            SignatureAlgorithm::SlhDsa128s => 7856,
            SignatureAlgorithm::SlhDsa128f => 17088,
            SignatureAlgorithm::SlhDsa192s => 16224,
            SignatureAlgorithm::SlhDsa192f => 35664,
            SignatureAlgorithm::SlhDsa256s => 29792,
            SignatureAlgorithm::SlhDsa256f => 49856,
        };

        vec![fill; len]
    }

    /// Constructs a deterministic signature entry with algorithm-compatible
    /// metadata. This helper prevents tests from failing for the wrong reason
    /// when the intent is to exercise policy, registry, or constitution logic.
    fn signature_entry(
        algorithm: SignatureAlgorithm,
        key_id: &str,
        fill: u8,
    ) -> SignatureEntry {
        SignatureEntry {
            algorithm,
            key_id: key_id.to_owned(),
            signature: valid_test_signature(algorithm, fill),
        }
    }

    #[test]
    fn verifier_resolves_latest_profile_version() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(9);

        let record = AuthProfileRecord {
            profile: AuthProfile::PostQuantumStrict,
            threshold: ThresholdPolicy {
                min_signers: 2,
                require_post_quantum: true,
            },
            quorum: QuorumPolicy {
                min_total: 2,
                min_governance: 1,
                min_operations: 0,
                min_system: 1,
            },
            signer_classes: BTreeMap::from([
                ("gov-1".to_owned(), SignerClass::Governance),
                ("sys-1".to_owned(), SignerClass::System),
            ]),
        };

        registry
            .insert_version(id, 1, record.clone())
            .expect("insert v1 must succeed");
        registry
            .insert_version(id, 2, record)
            .expect("insert v2 must succeed");

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![
                signature_entry(SignatureAlgorithm::MlDsa65, "gov-1", 0x11),
                signature_entry(SignatureAlgorithm::MlDsa87, "sys-1", 0x22),
            ],
        };

        let result = verify_envelope(
            &registry,
            id,
            None,
            &envelope,
            AuthEnvelopeLimits::default(),
        )
        .expect("verification should succeed for the latest registry version");

        assert_eq!(result.profile_version, 2);
        assert_eq!(result.signer_count, 2);
    }

    #[test]
    fn verifier_rejects_when_qrkf_lane_policy_fails() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(11);

        let record = AuthProfileRecord {
            profile: AuthProfile::PostQuantumStrict,
            threshold: ThresholdPolicy {
                min_signers: 1,
                require_post_quantum: true,
            },
            quorum: QuorumPolicy {
                min_total: 1,
                min_governance: 0,
                min_operations: 0,
                min_system: 0,
            },
            signer_classes: BTreeMap::from([("pq-1".to_owned(), SignerClass::Application)]),
        };

        registry
            .insert_version(id, 1, record)
            .expect("insert v1 must succeed");

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 8,
            signers: vec![signature_entry(SignatureAlgorithm::MlDsa87, "pq-1", 0x09)],
        };

        let bundle = EpochKeyBundle {
            realm: KeyRealm::Governance,
            profile_id: CryptoProfileId::AoxcPq1,
            epoch_id: 1,
            valid_from: 10,
            valid_until: 20,
            predecessor_fingerprint: None,
            rotation_reason: "bootstrap".to_owned(),
        };

        let err = verify_envelope_with_qrkf(
            &registry,
            id,
            Some(1),
            &envelope,
            AuthEnvelopeLimits::default(),
            QrkfVerification {
                lane_policy: LanePolicy {
                    min_approved_lanes: 2,
                    require_authority_lane: true,
                    require_continuity_lane: true,
                    require_recovery_veto_lane: false,
                },
                lane_approvals: &[AuthorizationLane::PostQuantumAuthenticity],
                current_bundle: &bundle,
                predecessor_bundle: None,
            },
        )
        .expect_err("verification should fail because QRKF lane policy is unsatisfied");

        assert!(
            err.to_string()
                .contains("qrkf failed: authorization lane policy not satisfied"),
            "unexpected QRKF failure reason: {err}"
        );
    }

    #[test]
    fn verifier_applies_constitution_by_domain_lane() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(12);

        let record = AuthProfileRecord {
            profile: AuthProfile::PostQuantumStrict,
            threshold: ThresholdPolicy {
                min_signers: 1,
                require_post_quantum: true,
            },
            quorum: QuorumPolicy {
                min_total: 1,
                min_governance: 0,
                min_operations: 0,
                min_system: 0,
            },
            signer_classes: BTreeMap::from([("pq-1".to_owned(), SignerClass::Application)]),
        };

        registry
            .insert_version(id, 1, record)
            .expect("insert v1 must succeed");

        let envelope = AuthEnvelope {
            domain: "AOX/TX/V1".to_owned(),
            nonce: 8,
            signers: vec![signature_entry(SignatureAlgorithm::MlDsa65, "pq-1", 0x33)],
        };

        let result = verify_envelope_under_constitution(
            &registry,
            id,
            Some(1),
            &envelope,
            AuthEnvelopeLimits::default(),
            &crate::auth::constitution::CryptographicConstitution::default(),
        );

        assert!(
            result.is_ok(),
            "constitution-aware verification should succeed for a valid operational domain"
        );
    }
}
