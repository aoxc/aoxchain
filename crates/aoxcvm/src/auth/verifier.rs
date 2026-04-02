//! Registry-backed auth envelope verification helpers.

use crate::{
    auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits},
        qrkf::{AuthorizationLane, EpochKeyBundle, LanePolicy},
        registry::{AuthProfileId, AuthProfileRegistry},
    },
    errors::AoxcvmError,
    errors::AoxcvmResult,
};

/// Deterministic verification result context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedAuthContext {
    pub profile_id: AuthProfileId,
    pub profile_version: u16,
    pub signer_count: usize,
}

/// Verifies an auth envelope against a canonical registry profile.
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
#[derive(Debug, Clone)]
pub struct QrkfVerification<'a> {
    pub lane_policy: LanePolicy,
    pub lane_approvals: &'a [AuthorizationLane],
    pub current_bundle: &'a EpochKeyBundle,
    pub predecessor_bundle: Option<&'a EpochKeyBundle>,
}

/// Verifies an envelope and enforces QRKF lane + continuity constraints.
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
        verifier::{verify_envelope, verify_envelope_with_qrkf, QrkfVerification},
    };

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
            .expect("insert v1");
        registry.insert_version(id, 2, record).expect("insert v2");

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa65,
                    key_id: "gov-1".to_owned(),
                    signature: vec![1_u8; 128],
                },
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa87,
                    key_id: "sys-1".to_owned(),
                    signature: vec![2_u8; 128],
                },
            ],
        };

        let result = verify_envelope(
            &registry,
            id,
            None,
            &envelope,
            AuthEnvelopeLimits::default(),
        )
        .expect("verification should pass");

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
        registry.insert_version(id, 1, record).expect("insert v1");

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 8,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa87,
                key_id: "pq-1".to_owned(),
                signature: vec![9_u8; 128],
            }],
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
        .expect_err("lane policy should fail");

        assert!(
            err.to_string()
                .contains("qrkf failed: authorization lane policy not satisfied")
        );
    }
}
