use std::collections::HashSet;

use crate::block::hash::{canonical_section_sort_key, compute_block_hash, compute_body_roots};
use crate::block::semantic::{
    validate_block_semantics, validate_capability_section_alignment,
    validate_root_semantic_bindings,
};
use crate::block::types::{
    BLOCK_VERSION_V1, Block, BlockBody, BlockBuildError, BlockHeader, BlockSection,
};

/// Deterministic block construction utility.
///
/// Design guarantees:
/// - block construction is fully deterministic for identical inputs,
/// - section ordering is canonicalized before root calculation,
/// - header hashing remains stable and compact,
/// - block construction does not depend on wall-clock side effects.
pub struct BlockBuilder;

impl BlockBuilder {
    /// Builds a canonical coordination block.
    ///
    /// The caller remains responsible for:
    /// - proposer selection,
    /// - timestamp policy enforcement,
    /// - parent validity,
    /// - lane-level semantic validation,
    /// - external proof verification.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        network_id: u32,
        parent_hash: [u8; 32],
        height: u64,
        era: u64,
        round: u64,
        timestamp: u64,
        proposer: [u8; 32],
        mut body: BlockBody,
    ) -> Result<Block, BlockBuildError> {
        if timestamp == 0 {
            return Err(BlockBuildError::ZeroTimestamp);
        }

        if proposer == [0u8; 32] {
            return Err(BlockBuildError::ZeroProposer);
        }

        let _ = u64::try_from(body.sections.len())
            .map_err(|_| BlockBuildError::SectionCountOverflow)?;

        canonicalize_body(&mut body)?;
        validate_block_semantics(timestamp, era, &body)?;

        let roots = compute_body_roots(&body);
        validate_capability_section_alignment(&body, roots.capability_flags)?;
        let empty_roots = compute_body_roots(&BlockBody::default());
        validate_root_semantic_bindings(&body, &roots, &empty_roots)?;

        let header = BlockHeader {
            version: BLOCK_VERSION_V1,
            network_id,
            parent_hash,
            height,
            era,
            round,
            timestamp,
            proposer,
            body_root: roots.body_root,
            finality_root: roots.finality_root,
            authority_root: roots.authority_root,
            lane_root: roots.lane_root,
            proof_root: roots.proof_root,
            identity_root: roots.identity_root,
            ai_root: roots.ai_root,
            pq_root: roots.pq_root,
            external_settlement_root: roots.external_settlement_root,
            policy_root: roots.policy_root,
            time_seal_root: roots.time_seal_root,
            capability_flags: roots.capability_flags,
            crypto_epoch: era,
        };

        let hash = compute_block_hash(&header);

        Ok(Block { hash, header, body })
    }
}

fn canonicalize_body(body: &mut BlockBody) -> Result<(), BlockBuildError> {
    let mut seen_section_types = HashSet::new();

    for section in &mut body.sections {
        if !seen_section_types.insert(section.discriminant()) {
            return Err(BlockBuildError::DuplicateSectionType);
        }

        match section {
            BlockSection::Execution(execution_section) => execution_section.lanes.sort(),
            BlockSection::LaneCommitment(lane_section) => lane_section.lanes.sort(),
            BlockSection::ExternalProof(proof_section) => proof_section.proofs.sort(),
            BlockSection::ExternalSettlement(settlement_section) => {
                settlement_section.settlements.sort()
            }
            BlockSection::Identity(_)
            | BlockSection::PostQuantum(_)
            | BlockSection::Ai(_)
            | BlockSection::Constitutional(_)
            | BlockSection::TimeSeal(_) => {}
        }
    }

    body.sections.sort_by_key(canonical_section_sort_key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::block::types::{
        AiSection, BlockBody, BlockBuildError, BlockSection, CAPABILITY_AI_ATTESTATION,
        CAPABILITY_EXECUTION, CAPABILITY_SETTLEMENT, ExternalNetwork, ExternalProofRecord,
        ExternalProofSection, ExternalProofType, IdentitySection, LaneCommitment,
        LaneCommitmentSection, LaneType, PostQuantumSection, TimeSealSection,
    };

    use super::BlockBuilder;

    #[test]
    fn builds_deterministic_block_for_identical_inputs() {
        let body = BlockBody {
            sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
                lanes: vec![LaneCommitment {
                    lane_id: 1,
                    lane_type: LaneType::Native,
                    tx_count: 2,
                    input_root: [1u8; 32],
                    output_root: [2u8; 32],
                    receipt_root: [3u8; 32],
                    state_commitment: [4u8; 32],
                    proof_commitment: [5u8; 32],
                }],
            })],
        };

        let block_a = BlockBuilder::build(
            1,
            [9u8; 32],
            10,
            0,
            7,
            1_735_689_600,
            [8u8; 32],
            body.clone(),
        )
        .unwrap();

        let block_b =
            BlockBuilder::build(1, [9u8; 32], 10, 0, 7, 1_735_689_600, [8u8; 32], body).unwrap();

        assert_eq!(block_a, block_b);
    }

    #[test]
    fn canonicalizes_section_order() {
        let lane_section = BlockSection::LaneCommitment(LaneCommitmentSection {
            lanes: vec![LaneCommitment {
                lane_id: 7,
                lane_type: LaneType::Evm,
                tx_count: 3,
                input_root: [1u8; 32],
                output_root: [2u8; 32],
                receipt_root: [3u8; 32],
                state_commitment: [4u8; 32],
                proof_commitment: [5u8; 32],
            }],
        });

        let proof_section = BlockSection::ExternalProof(ExternalProofSection {
            proofs: vec![ExternalProofRecord {
                source_network: ExternalNetwork::Sui,
                proof_type: ExternalProofType::Finality,
                subject_hash: [6u8; 32],
                proof_commitment: [7u8; 32],
                finalized_at: 1_735_689_600,
            }],
        });

        let block_a = BlockBuilder::build(
            1,
            [1u8; 32],
            1,
            0,
            1,
            1_735_689_600,
            [2u8; 32],
            BlockBody {
                sections: vec![lane_section.clone(), proof_section.clone()],
            },
        )
        .unwrap();

        let block_b = BlockBuilder::build(
            1,
            [1u8; 32],
            1,
            0,
            1,
            1_735_689_600,
            [2u8; 32],
            BlockBody {
                sections: vec![proof_section, lane_section],
            },
        )
        .unwrap();

        assert_eq!(block_a.hash, block_b.hash);
        assert_eq!(block_a.header.body_root, block_b.header.body_root);
    }

    #[test]
    fn canonicalizes_nested_lane_and_proof_ordering() {
        let body_a = BlockBody {
            sections: vec![
                BlockSection::LaneCommitment(LaneCommitmentSection {
                    lanes: vec![
                        LaneCommitment {
                            lane_id: 9,
                            lane_type: LaneType::Wasm,
                            tx_count: 4,
                            input_root: [1u8; 32],
                            output_root: [2u8; 32],
                            receipt_root: [3u8; 32],
                            state_commitment: [4u8; 32],
                            proof_commitment: [5u8; 32],
                        },
                        LaneCommitment {
                            lane_id: 1,
                            lane_type: LaneType::Native,
                            tx_count: 1,
                            input_root: [6u8; 32],
                            output_root: [7u8; 32],
                            receipt_root: [8u8; 32],
                            state_commitment: [9u8; 32],
                            proof_commitment: [10u8; 32],
                        },
                    ],
                }),
                BlockSection::ExternalProof(ExternalProofSection {
                    proofs: vec![
                        ExternalProofRecord {
                            source_network: ExternalNetwork::Sui,
                            proof_type: ExternalProofType::Checkpoint,
                            subject_hash: [11u8; 32],
                            proof_commitment: [12u8; 32],
                            finalized_at: 10,
                        },
                        ExternalProofRecord {
                            source_network: ExternalNetwork::Bitcoin,
                            proof_type: ExternalProofType::Finality,
                            subject_hash: [13u8; 32],
                            proof_commitment: [14u8; 32],
                            finalized_at: 11,
                        },
                    ],
                }),
            ],
        };

        let body_b = BlockBody {
            sections: vec![
                BlockSection::ExternalProof(ExternalProofSection {
                    proofs: vec![
                        ExternalProofRecord {
                            source_network: ExternalNetwork::Bitcoin,
                            proof_type: ExternalProofType::Finality,
                            subject_hash: [13u8; 32],
                            proof_commitment: [14u8; 32],
                            finalized_at: 11,
                        },
                        ExternalProofRecord {
                            source_network: ExternalNetwork::Sui,
                            proof_type: ExternalProofType::Checkpoint,
                            subject_hash: [11u8; 32],
                            proof_commitment: [12u8; 32],
                            finalized_at: 10,
                        },
                    ],
                }),
                BlockSection::LaneCommitment(LaneCommitmentSection {
                    lanes: vec![
                        LaneCommitment {
                            lane_id: 1,
                            lane_type: LaneType::Native,
                            tx_count: 1,
                            input_root: [6u8; 32],
                            output_root: [7u8; 32],
                            receipt_root: [8u8; 32],
                            state_commitment: [9u8; 32],
                            proof_commitment: [10u8; 32],
                        },
                        LaneCommitment {
                            lane_id: 9,
                            lane_type: LaneType::Wasm,
                            tx_count: 4,
                            input_root: [1u8; 32],
                            output_root: [2u8; 32],
                            receipt_root: [3u8; 32],
                            state_commitment: [4u8; 32],
                            proof_commitment: [5u8; 32],
                        },
                    ],
                }),
            ],
        };

        let block_a = BlockBuilder::build(1, [1u8; 32], 2, 0, 2, 100, [3u8; 32], body_a).unwrap();
        let block_b = BlockBuilder::build(1, [1u8; 32], 2, 0, 2, 100, [3u8; 32], body_b).unwrap();

        assert_eq!(block_a.hash, block_b.hash);
        assert_eq!(block_a.header.body_root, block_b.header.body_root);
    }

    #[test]
    fn rejects_duplicate_section_types() {
        let body = BlockBody {
            sections: vec![
                BlockSection::LaneCommitment(LaneCommitmentSection::default()),
                BlockSection::LaneCommitment(LaneCommitmentSection::default()),
            ],
        };

        let err = BlockBuilder::build(1, [0u8; 32], 0, 0, 0, 1, [7u8; 32], body).unwrap_err();
        assert_eq!(err, BlockBuildError::DuplicateSectionType);
    }

    #[test]
    fn validates_time_seal_window() {
        let error = BlockBuilder::build(
            1,
            [0u8; 32],
            1,
            1,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![BlockSection::TimeSeal(TimeSealSection {
                    valid_from: 200,
                    valid_until: 300,
                    epoch_action_root: [1u8; 32],
                    delayed_effect_root: [2u8; 32],
                })],
            },
        )
        .unwrap_err();

        assert_eq!(error, BlockBuildError::TimestampOutsideTimeSealWindow);
    }

    #[test]
    fn validates_ai_policy_and_nonce() {
        let error = BlockBuilder::build(
            1,
            [0u8; 32],
            1,
            1,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![BlockSection::Ai(AiSection {
                    request_hash: [1u8; 32],
                    response_hash: [2u8; 32],
                    policy_hash: [0u8; 32],
                    confidence_commitment: [3u8; 32],
                    human_override: false,
                    fallback_mode: false,
                    replay_nonce: 0,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(error, BlockBuildError::AiSectionMissingPolicyHash);
    }

    #[test]
    fn validates_post_quantum_signature_policy_id() {
        let error = BlockBuilder::build(
            1,
            [0u8; 32],
            1,
            1,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                    scheme_registry_root: [1u8; 32],
                    signer_set_root: [2u8; 32],
                    hybrid_policy_root: [3u8; 32],
                    signature_policy_id: 0,
                    downgrade_prohibited: true,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(error, BlockBuildError::PostQuantumMissingSignaturePolicy);
    }

    #[test]
    fn enforces_pq_mandatory_policy_after_migration_epoch() {
        let error = BlockBuilder::build(
            1,
            [0u8; 32],
            1,
            100,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                    scheme_registry_root: [1u8; 32],
                    signer_set_root: [2u8; 32],
                    hybrid_policy_root: [3u8; 32],
                    signature_policy_id: 2,
                    downgrade_prohibited: true,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(error, BlockBuildError::CryptoEpochRequiresPqMandatory);
    }

    #[test]
    fn pq_mandatory_requires_downgrade_protection() {
        let error = BlockBuilder::build(
            1,
            [0u8; 32],
            1,
            120,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                    scheme_registry_root: [1u8; 32],
                    signer_set_root: [2u8; 32],
                    hybrid_policy_root: [3u8; 32],
                    signature_policy_id: 4,
                    downgrade_prohibited: false,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(
            error,
            BlockBuildError::PqMandatoryRequiresDowngradeProtection
        );
    }

    #[test]
    fn header_capabilities_are_exposed_via_helper() {
        let block = BlockBuilder::build(
            1,
            [0u8; 32],
            3,
            2,
            1,
            100,
            [1u8; 32],
            BlockBody {
                sections: vec![
                    BlockSection::LaneCommitment(LaneCommitmentSection {
                        lanes: vec![LaneCommitment {
                            lane_id: 1,
                            lane_type: LaneType::Native,
                            tx_count: 1,
                            input_root: [1u8; 32],
                            output_root: [2u8; 32],
                            receipt_root: [3u8; 32],
                            state_commitment: [4u8; 32],
                            proof_commitment: [5u8; 32],
                        }],
                    }),
                    BlockSection::ExternalProof(ExternalProofSection {
                        proofs: vec![ExternalProofRecord {
                            source_network: ExternalNetwork::Ethereum,
                            proof_type: ExternalProofType::Finality,
                            subject_hash: [6u8; 32],
                            proof_commitment: [7u8; 32],
                            finalized_at: 99,
                        }],
                    }),
                    BlockSection::Ai(AiSection {
                        request_hash: [10u8; 32],
                        response_hash: [11u8; 32],
                        policy_hash: [12u8; 32],
                        confidence_commitment: [13u8; 32],
                        human_override: false,
                        fallback_mode: false,
                        replay_nonce: 1,
                    }),
                ],
            },
        )
        .unwrap();

        assert!(block.header.has_capability(CAPABILITY_EXECUTION));
        assert!(block.header.has_capability(CAPABILITY_SETTLEMENT));
        assert!(block.header.has_capability(CAPABILITY_AI_ATTESTATION));
    }

    #[test]
    fn full_constitutional_block_sets_all_expected_roots_and_capabilities() {
        let block = BlockBuilder::build(
            1,
            [1u8; 32],
            11,
            120,
            4,
            200,
            [9u8; 32],
            BlockBody {
                sections: vec![
                    BlockSection::LaneCommitment(LaneCommitmentSection {
                        lanes: vec![LaneCommitment {
                            lane_id: 2,
                            lane_type: LaneType::Evm,
                            tx_count: 3,
                            input_root: [2u8; 32],
                            output_root: [3u8; 32],
                            receipt_root: [4u8; 32],
                            state_commitment: [5u8; 32],
                            proof_commitment: [6u8; 32],
                        }],
                    }),
                    BlockSection::Identity(IdentitySection {
                        validator_snapshot_root: [11u8; 32],
                        session_keys_root: [12u8; 32],
                        revocation_root: [13u8; 32],
                        authority_epoch_proof: [14u8; 32],
                    }),
                    BlockSection::PostQuantum(PostQuantumSection {
                        scheme_registry_root: [15u8; 32],
                        signer_set_root: [16u8; 32],
                        hybrid_policy_root: [17u8; 32],
                        signature_policy_id: 4,
                        downgrade_prohibited: true,
                    }),
                    BlockSection::Ai(AiSection {
                        request_hash: [18u8; 32],
                        response_hash: [19u8; 32],
                        policy_hash: [20u8; 32],
                        confidence_commitment: [21u8; 32],
                        human_override: false,
                        fallback_mode: false,
                        replay_nonce: 7,
                    }),
                    BlockSection::ExternalProof(ExternalProofSection {
                        proofs: vec![ExternalProofRecord {
                            source_network: ExternalNetwork::Ethereum,
                            proof_type: ExternalProofType::Finality,
                            subject_hash: [22u8; 32],
                            proof_commitment: [23u8; 32],
                            finalized_at: 123,
                        }],
                    }),
                    BlockSection::Constitutional(Default::default()),
                    BlockSection::TimeSeal(TimeSealSection {
                        valid_from: 100,
                        valid_until: 300,
                        epoch_action_root: [24u8; 32],
                        delayed_effect_root: [25u8; 32],
                    }),
                ],
            },
        )
        .unwrap();

        assert_ne!(block.header.policy_root, [0u8; 32]);
        assert_ne!(block.header.authority_root, [0u8; 32]);
        assert!(block.header.has_capability(CAPABILITY_EXECUTION));
        assert!(block.header.has_capability(CAPABILITY_SETTLEMENT));
        assert!(block.header.has_capability(CAPABILITY_AI_ATTESTATION));
    }
}
