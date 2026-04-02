//! Registry-backed auth envelope verification helpers.

use crate::{
    auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits},
        registry::{AuthProfileId, AuthProfileRegistry},
    },
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry},
        policy::QuorumPolicy,
        registry::{AuthProfileId, AuthProfileRecord, AuthProfileRegistry},
        scheme::{AuthProfile, SignatureAlgorithm},
        signer::SignerClass,
        threshold::ThresholdPolicy,
        verifier::verify_envelope,
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
}
