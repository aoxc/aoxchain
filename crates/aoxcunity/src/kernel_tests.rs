#[cfg(test)]
mod tests {
    use crate::block::{Block, BlockBody, BlockBuilder};
    use crate::constitutional::LegitimacyCertificate;
    use crate::quorum::QuorumThreshold;
    use crate::rotation::ValidatorRotation;
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::{
        ConsensusEngine, ConsensusEvent, InvariantStatus, KernelCertificate, KernelEffect,
        KernelRejection, TimeoutVote, TransitionResult, VerifiedTimeoutVote, VerifiedVote,
    };

    fn validator(id: u8, power: u64) -> Validator {
        Validator::new([id; 32], power, ValidatorRole::Validator)
    }

    fn engine() -> ConsensusEngine {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 4), validator(2, 3), validator(3, 3)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum))
    }

    fn vote_context(engine: &ConsensusEngine, epoch: u64) -> VoteAuthenticationContext {
        VoteAuthenticationContext {
            network_id: 2626,
            epoch,
            validator_set_root: engine.state.rotation.validator_set_hash(),
            pq_attestation_root: engine.state.rotation.pq_attestation_root(),
            signature_scheme: 1,
        }
    }

    fn make_block(parent_hash: [u8; 32], height: u64, proposer: [u8; 32], round: u64) -> Block {
        BlockBuilder::build(
            1,
            parent_hash,
            height,
            round,
            height,
            height + 1,
            proposer,
            BlockBody::default(),
        )
        .unwrap()
    }

    fn commit_vote(
        engine: &ConsensusEngine,
        voter: u8,
        block: &Block,
        round: u64,
    ) -> ConsensusEvent {
        ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
            authenticated_vote: VerifiedAuthenticatedVote {
                vote: Vote {
                    voter: [voter; 32],
                    block_hash: block.hash,
                    height: block.header.height,
                    round,
                    kind: VoteKind::Commit,
                },
                context: vote_context(engine, 0),
            },
            verification_tag: [voter.wrapping_add(20); 32],
        })
    }

    #[test]
    fn accepted_transition_result_is_explicit_and_healthy() {
        let result = TransitionResult::accepted(KernelEffect::BlockAccepted([1u8; 32]));

        assert_eq!(result.accepted_effects.len(), 1);
        assert!(result.rejected_reason.is_none());
        assert_eq!(result.invariant_status, InvariantStatus::healthy());
    }

    #[test]
    fn rejected_transition_result_carries_explicit_reason() {
        let result = TransitionResult::rejected(KernelRejection::StaleArtifact);

        assert!(result.accepted_effects.is_empty());
        assert_eq!(result.rejected_reason, Some(KernelRejection::StaleArtifact));
    }

    #[test]
    fn deterministic_event_stream_produces_same_results_and_state() {
        let mut a = engine();
        let mut b = engine();

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let child = make_block(genesis.hash, 1, [2u8; 32], 1);

        let events = vec![
            ConsensusEvent::AdmitBlock(genesis.clone()),
            ConsensusEvent::AdmitBlock(child.clone()),
            commit_vote(&a, 1, &child, 1),
            commit_vote(&a, 2, &child, 1),
            commit_vote(&a, 3, &child, 1),
            ConsensusEvent::EvaluateFinality {
                block_hash: child.hash,
            },
        ];

        let results_a: Vec<_> = events
            .iter()
            .cloned()
            .map(|event| a.apply_event(event))
            .collect();

        let results_b: Vec<_> = events
            .into_iter()
            .map(|event| b.apply_event(event))
            .collect();

        assert_eq!(results_a, results_b);
        assert_eq!(
            a.state.fork_choice.finalized_head(),
            b.state.fork_choice.finalized_head()
        );
        assert_eq!(a.state.round, b.state.round);
        assert_eq!(a.lock_state, b.lock_state);
    }

    #[test]
    fn timeout_quorum_emits_continuity_certificate_and_advances_round() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let timeout_event = |voter: u8| {
            ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash: block.hash,
                    height: 1,
                    round: 1,
                    epoch: 4,
                    timeout_round: 2,
                    voter: [voter; 32],
                },
                verification_tag: [voter.wrapping_add(10); 32],
            })
        };

        assert!(
            engine
                .apply_event(timeout_event(1))
                .emitted_certificates
                .is_empty()
        );

        let result = engine.apply_event(timeout_event(2));

        assert!(
            result
                .accepted_effects
                .contains(&KernelEffect::RoundAdvanced {
                    height: 1,
                    round: 3
                })
        );
        assert!(
            result
                .emitted_certificates
                .iter()
                .any(|certificate| matches!(certificate, KernelCertificate::Continuity(_)))
        );
        assert_eq!(engine.state.round.round, 3);
    }

    #[test]
    fn finality_completes_when_legitimacy_and_continuity_exist() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let _ = engine.apply_event(ConsensusEvent::ObserveLegitimacy(
            LegitimacyCertificate::new(
                block.hash,
                0,
                [1u8; 32],
                [2u8; 32],
                [3u8; 32],
                vec![[1u8; 32], [2u8; 32], [3u8; 32]],
            ),
        ));
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash: block.hash,
                    height: 1,
                    round: 1,
                    epoch: 0,
                    timeout_round: 1,
                    voter: [voter; 32],
                },
                verification_tag: [voter; 32],
            }));
        }
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(commit_vote(&engine, voter, &block, 1));
        }

        let result = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });

        assert!(result.rejected_reason.is_none());
        assert_eq!(
            result.accepted_effects,
            vec![KernelEffect::BlockFinalized(block.hash)]
        );
    }

    #[test]
    fn invalid_legitimacy_certificate_is_rejected() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let result = engine.apply_event(ConsensusEvent::ObserveLegitimacy(
            LegitimacyCertificate::new(block.hash, 0, [1u8; 32], [2u8; 32], [3u8; 32], vec![]),
        ));

        assert_eq!(
            result.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert!(result.invariant_status.conflicting_finality_detected);
    }

    #[test]
    fn custom_crypto_profile_is_used_for_execution_certificate_context() {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 4), validator(2, 3), validator(3, 3)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        let state = crate::state::ConsensusState::new(rotation, quorum);
        let mut engine = ConsensusEngine::with_crypto_profile(state, 4040, 42);

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
                authenticated_vote: VerifiedAuthenticatedVote {
                    vote: Vote {
                        voter: [voter; 32],
                        block_hash: block.hash,
                        height: block.header.height,
                        round: 1,
                        kind: VoteKind::Commit,
                    },
                    context: VoteAuthenticationContext {
                        network_id: 4040,
                        epoch: 0,
                        validator_set_root: engine.state.rotation.validator_set_hash(),
                        pq_attestation_root: engine.state.rotation.pq_attestation_root(),
                        signature_scheme: 42,
                    },
                },
                verification_tag: [voter; 32],
            }));
        }

        let result = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });

        let execution = result
            .emitted_certificates
            .iter()
            .find_map(|certificate| match certificate {
                KernelCertificate::Execution(certificate) => Some(certificate),
                _ => None,
            })
            .expect("execution certificate should be emitted");

        assert_eq!(execution.network_id, 4040);
        assert_eq!(execution.signature_scheme, 42);
    }

    #[test]
    fn duplicate_recovery_event_sets_replay_diverged_invariant() {
        let mut engine = engine();
        let event_hash = [0xAA; 32];

        let first = engine.apply_event(ConsensusEvent::RecoverPersistedEvent { event_hash });
        let second = engine.apply_event(ConsensusEvent::RecoverPersistedEvent { event_hash });

        assert_eq!(
            first.accepted_effects,
            vec![KernelEffect::StateRecovered(event_hash)]
        );
        assert_eq!(
            second.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
        assert!(second.invariant_status.replay_diverged);
    }

    #[test]
    fn equal_height_sibling_admission_preserves_live_fork_tracking() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let canonical = make_block(genesis.hash, 1, [2u8; 32], 1);
        let conflicting = make_block(genesis.hash, 1, [3u8; 32], 1);
        let canonical_hash = canonical.hash;
        let conflicting_hash = conflicting.hash;

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(canonical));

        let result = engine.apply_event(ConsensusEvent::AdmitBlock(conflicting));

        assert_eq!(result.rejected_reason, None);
        assert!(!result.invariant_status.stale_branch_reactivated);
        assert_eq!(
            engine.state.fork_choice.get_head(),
            Some(canonical_hash.max(conflicting_hash))
        );
    }

    #[test]
    fn timeout_equivocation_sets_conflicting_finality_invariant() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block_a = make_block(genesis.hash, 1, [2u8; 32], 1);
        let block_b = Block {
            hash: [0xBB; 32],
            ..make_block(genesis.hash, 1, [3u8; 32], 1)
        };

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block_a.clone()));

        engine.state.blocks.insert(block_b.hash, block_b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block_b.hash,
                parent: block_b.header.parent_hash,
                height: block_b.header.height,
                seal: None,
            });

        let vote = |block_hash| {
            ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash,
                    height: 1,
                    round: 1,
                    epoch: 0,
                    timeout_round: 2,
                    voter: [1u8; 32],
                },
                verification_tag: [8u8; 32],
            })
        };

        let first = engine.apply_event(vote(block_a.hash));
        let second = engine.apply_event(vote(block_b.hash));

        assert_eq!(
            first.accepted_effects,
            vec![KernelEffect::TimeoutAccepted(block_a.hash)]
        );
        assert_eq!(
            second.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert!(second.invariant_status.conflicting_finality_detected);
    }
}
