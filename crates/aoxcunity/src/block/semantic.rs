use crate::block::hash::BodyRoots;
use crate::block::policy_registry::{enforce_signature_policy_migration, resolve_signature_policy};
use crate::block::types::{
    BlockBody, BlockBuildError, BlockSection, CAPABILITY_AI_ATTESTATION, CAPABILITY_CONSTITUTIONAL,
    CAPABILITY_EXECUTION, CAPABILITY_IDENTITY, CAPABILITY_PQ_ROTATION, CAPABILITY_SETTLEMENT,
    CAPABILITY_TIME_SEAL, PostQuantumSection, TimeSealSection,
};

/// Validates section-level semantic invariants for a canonical block body.
///
/// This layer is intentionally independent from hashing/canonical ordering so
/// policy checks can evolve without destabilizing deterministic hash behavior.
pub fn validate_block_semantics(
    timestamp: u64,
    crypto_epoch: u64,
    body: &BlockBody,
) -> Result<(), BlockBuildError> {
    let mut time_seal: Option<&TimeSealSection> = None;
    let mut pq_section: Option<&PostQuantumSection> = None;
    let mut ai_section_present = false;
    let mut ai_policy_hash = [0u8; 32];
    let mut ai_replay_nonce = 0u64;

    for section in &body.sections {
        match section {
            BlockSection::TimeSeal(section) => time_seal = Some(section),
            BlockSection::PostQuantum(section) => pq_section = Some(section),
            BlockSection::Ai(section) => {
                ai_section_present = true;
                ai_policy_hash = section.policy_hash;
                ai_replay_nonce = section.replay_nonce;
            }
            _ => {}
        }
    }

    if let Some(section) = time_seal {
        validate_time_seal(timestamp, section)?;
    }

    if ai_section_present {
        if ai_policy_hash == [0u8; 32] {
            return Err(BlockBuildError::AiSectionMissingPolicyHash);
        }

        if ai_replay_nonce == 0 {
            return Err(BlockBuildError::AiSectionZeroReplayNonce);
        }
    }

    if let Some(section) = pq_section {
        validate_post_quantum_policy(crypto_epoch, section)?;
    }

    Ok(())
}

fn validate_time_seal(timestamp: u64, section: &TimeSealSection) -> Result<(), BlockBuildError> {
    if section.valid_from > section.valid_until {
        return Err(BlockBuildError::InvalidTimeSealRange);
    }

    if timestamp < section.valid_from || timestamp > section.valid_until {
        return Err(BlockBuildError::TimestampOutsideTimeSealWindow);
    }

    Ok(())
}

fn validate_post_quantum_policy(
    crypto_epoch: u64,
    section: &PostQuantumSection,
) -> Result<(), BlockBuildError> {
    let policy = resolve_signature_policy(section.signature_policy_id)?;
    enforce_signature_policy_migration(crypto_epoch, policy, section.downgrade_prohibited)
}

pub fn validate_capability_section_alignment(
    body: &BlockBody,
    capability_flags: u64,
) -> Result<(), BlockBuildError> {
    let mut expected = 0u64;
    for section in &body.sections {
        match section {
            BlockSection::Execution(_) | BlockSection::LaneCommitment(_) => {
                expected |= CAPABILITY_EXECUTION;
            }
            BlockSection::Identity(_) => expected |= CAPABILITY_IDENTITY,
            BlockSection::ExternalProof(_) | BlockSection::ExternalSettlement(_) => {
                expected |= CAPABILITY_SETTLEMENT;
            }
            BlockSection::Ai(_) => expected |= CAPABILITY_AI_ATTESTATION,
            BlockSection::PostQuantum(_) => expected |= CAPABILITY_PQ_ROTATION,
            BlockSection::Constitutional(_) => expected |= CAPABILITY_CONSTITUTIONAL,
            BlockSection::TimeSeal(_) => expected |= CAPABILITY_TIME_SEAL,
        }
    }

    if expected != capability_flags {
        return Err(BlockBuildError::CapabilitySectionMismatch);
    }

    Ok(())
}

pub fn validate_root_semantic_bindings(
    body: &BlockBody,
    roots: &BodyRoots,
    empty_roots: &BodyRoots,
) -> Result<(), BlockBuildError> {
    let mut requires_policy_root = false;
    let mut requires_authority_root = false;

    for section in &body.sections {
        match section {
            BlockSection::Ai(_)
            | BlockSection::PostQuantum(_)
            | BlockSection::Constitutional(_) => {
                requires_policy_root = true;
            }
            _ => {}
        }

        match section {
            BlockSection::Identity(_) | BlockSection::Constitutional(_) => {
                requires_authority_root = true;
            }
            _ => {}
        }
    }

    if requires_policy_root && roots.policy_root == empty_roots.policy_root {
        return Err(BlockBuildError::PolicyRootBindingMismatch);
    }

    if requires_authority_root && roots.authority_root == empty_roots.authority_root {
        return Err(BlockBuildError::AuthorityRootBindingMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_block_semantics, validate_capability_section_alignment,
        validate_root_semantic_bindings,
    };
    use crate::block::hash::compute_body_roots;
    use crate::block::types::{
        AiSection, BlockBody, BlockBuildError, BlockSection, CAPABILITY_AI_ATTESTATION,
        CAPABILITY_CONSTITUTIONAL, CAPABILITY_EXECUTION, CAPABILITY_IDENTITY,
        CAPABILITY_PQ_ROTATION, CAPABILITY_SETTLEMENT, CAPABILITY_TIME_SEAL, IdentitySection,
        PostQuantumSection, TimeSealSection,
    };

    #[test]
    fn rejects_unsupported_signature_policy_id() {
        let err = validate_block_semantics(
            10,
            1,
            &BlockBody {
                sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                    scheme_registry_root: [1u8; 32],
                    signer_set_root: [2u8; 32],
                    hybrid_policy_root: [3u8; 32],
                    signature_policy_id: 9,
                    downgrade_prohibited: true,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(err, BlockBuildError::PostQuantumInvalidSignaturePolicy);
    }

    #[test]
    fn rejects_capability_section_mismatch() {
        let err = validate_capability_section_alignment(
            &BlockBody {
                sections: vec![BlockSection::TimeSeal(TimeSealSection {
                    valid_from: 1,
                    valid_until: 2,
                    epoch_action_root: [0u8; 32],
                    delayed_effect_root: [0u8; 32],
                })],
            },
            CAPABILITY_EXECUTION,
        )
        .unwrap_err();

        assert_eq!(err, BlockBuildError::CapabilitySectionMismatch);
    }

    #[test]
    fn root_semantic_bindings_accept_bound_roots() {
        let body = BlockBody {
            sections: vec![BlockSection::Identity(Default::default())],
        };
        let roots = compute_body_roots(&body);
        let empty = compute_body_roots(&BlockBody::default());

        let result = validate_root_semantic_bindings(&body, &roots, &empty);
        assert!(result.is_ok());
    }

    #[test]
    fn root_semantic_bindings_reject_policy_root_mismatch() {
        let body = BlockBody {
            sections: vec![BlockSection::Ai(AiSection {
                request_hash: [1u8; 32],
                response_hash: [2u8; 32],
                policy_hash: [3u8; 32],
                confidence_commitment: [4u8; 32],
                human_override: false,
                fallback_mode: false,
                replay_nonce: 1,
            })],
        };

        let empty_roots = compute_body_roots(&BlockBody::default());
        let err = validate_root_semantic_bindings(&body, &empty_roots, &empty_roots).unwrap_err();
        assert_eq!(err, BlockBuildError::PolicyRootBindingMismatch);
    }

    #[test]
    fn root_semantic_bindings_reject_authority_root_mismatch() {
        let body = BlockBody {
            sections: vec![BlockSection::Identity(IdentitySection::default())],
        };

        let empty_roots = compute_body_roots(&BlockBody::default());
        let err = validate_root_semantic_bindings(&body, &empty_roots, &empty_roots).unwrap_err();
        assert_eq!(err, BlockBuildError::AuthorityRootBindingMismatch);
    }

    #[test]
    fn capability_alignment_matrix_matches_expected_flags() {
        let cases = vec![
            (
                BlockBody {
                    sections: vec![BlockSection::LaneCommitment(Default::default())],
                },
                CAPABILITY_EXECUTION,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::Identity(IdentitySection::default())],
                },
                CAPABILITY_IDENTITY,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::ExternalProof(Default::default())],
                },
                CAPABILITY_SETTLEMENT,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::Ai(AiSection {
                        request_hash: [1u8; 32],
                        response_hash: [2u8; 32],
                        policy_hash: [3u8; 32],
                        confidence_commitment: [4u8; 32],
                        human_override: false,
                        fallback_mode: false,
                        replay_nonce: 1,
                    })],
                },
                CAPABILITY_AI_ATTESTATION,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                        scheme_registry_root: [1u8; 32],
                        signer_set_root: [2u8; 32],
                        hybrid_policy_root: [3u8; 32],
                        signature_policy_id: 4,
                        downgrade_prohibited: true,
                    })],
                },
                CAPABILITY_PQ_ROTATION,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::Constitutional(Default::default())],
                },
                CAPABILITY_CONSTITUTIONAL,
            ),
            (
                BlockBody {
                    sections: vec![BlockSection::TimeSeal(TimeSealSection {
                        valid_from: 1,
                        valid_until: 2,
                        epoch_action_root: [0u8; 32],
                        delayed_effect_root: [0u8; 32],
                    })],
                },
                CAPABILITY_TIME_SEAL,
            ),
            (
                BlockBody {
                    sections: vec![
                        BlockSection::LaneCommitment(Default::default()),
                        BlockSection::Ai(AiSection {
                            request_hash: [1u8; 32],
                            response_hash: [2u8; 32],
                            policy_hash: [3u8; 32],
                            confidence_commitment: [4u8; 32],
                            human_override: false,
                            fallback_mode: false,
                            replay_nonce: 1,
                        }),
                        BlockSection::PostQuantum(PostQuantumSection {
                            scheme_registry_root: [1u8; 32],
                            signer_set_root: [2u8; 32],
                            hybrid_policy_root: [3u8; 32],
                            signature_policy_id: 4,
                            downgrade_prohibited: true,
                        }),
                    ],
                },
                CAPABILITY_EXECUTION | CAPABILITY_AI_ATTESTATION | CAPABILITY_PQ_ROTATION,
            ),
        ];

        for (body, expected_flags) in cases {
            let res = validate_capability_section_alignment(&body, expected_flags);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn empty_body_has_zero_capability_and_no_root_binding_requirement() {
        let body = BlockBody::default();
        let roots = compute_body_roots(&body);
        let empty = compute_body_roots(&BlockBody::default());

        assert!(validate_capability_section_alignment(&body, 0).is_ok());
        assert!(validate_root_semantic_bindings(&body, &roots, &empty).is_ok());
    }
}
