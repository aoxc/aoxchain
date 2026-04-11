#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

    use crate::block::{BlockBody, BlockBuilder};
    use crate::error::ConsensusError;
    use crate::quorum::QuorumThreshold;
    use crate::rotation::ValidatorRotation;
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::ConsensusState;

    fn validator(id: u8, power: u64, role: ValidatorRole, active: bool) -> Validator {
        let mut validator = Validator::new([id; 32], power, role);
        validator.active = active;
        validator
    }

    fn state_with_validators(validators: Vec<Validator>) -> ConsensusState {
        let rotation = ValidatorRotation::new(validators).unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        ConsensusState::new(rotation, quorum)
    }

    fn make_block(parent_hash: [u8; 32], height: u64, proposer: [u8; 32]) -> crate::block::Block {
        BlockBuilder::build(
            1,
            parent_hash,
            height,
            0,
            height,
            height + 1,
            proposer,
            BlockBody::default(),
        )
        .unwrap()
    }

    fn admit_genesis(state: &mut ConsensusState, proposer: [u8; 32]) -> crate::block::Block {
        let genesis = make_block([0u8; 32], 0, proposer);
        state.admit_block(genesis.clone()).unwrap();
        genesis
    }

    fn inject_known_block(state: &mut ConsensusState, block: crate::block::Block) {
        state.blocks.insert(block.hash, block.clone());
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block.hash,
                parent: block.header.parent_hash,
                height: block.header.height,
                seal: None,
            });
    }

    #[test]
    fn rejects_vote_from_unknown_validator() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let err = state
            .add_vote(Vote {
                voter: [9u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::ValidatorNotFound.to_string()
        );
    }

    #[test]
    fn rejects_vote_from_inactive_validator() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, false)]);
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::InactiveValidator.to_string()
        );
    }

    #[test]
    fn rejects_vote_from_non_voting_role() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Observer, true)]);
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::NonVotingValidator.to_string()
        );
    }

    #[test]
    fn observed_voting_power_ignores_inactive_and_non_eligible_validators() {
        let validators = vec![
            validator(1, 10, ValidatorRole::Validator, true),
            validator(2, 25, ValidatorRole::Observer, true),
            validator(3, 30, ValidatorRole::Validator, false),
        ];
        let rotation = ValidatorRotation::new(validators).unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        state
            .vote_pool
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();
        state
            .vote_pool
            .add_vote(Vote {
                voter: [2u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();
        state
            .vote_pool
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert_eq!(
            state.observed_voting_power(block.hash, VoteKind::Commit),
            10
        );
    }

    #[test]
    fn duplicate_vote_is_rejected_as_duplicate() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);
        let vote = Vote {
            voter: [1u8; 32],
            block_hash: block.hash,
            height: 0,
            round: 1,
            kind: VoteKind::Prepare,
        };

        state.add_vote(vote.clone()).unwrap();
        let err = state.add_vote(vote).unwrap_err();

        assert_eq!(err.to_string(), ConsensusError::DuplicateVote.to_string());
    }

    #[test]
    fn conflicting_same_round_vote_is_rejected_as_equivocation() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let child_a = make_block(genesis.hash, 1, [1u8; 32]);
        let child_b = make_block(genesis.hash, 1, [1u8; 32]);

        state.admit_block(child_a.clone()).unwrap();
        state.admit_block(child_b.clone()).unwrap_err();

        let alt_hash = [9u8; 32];
        inject_known_block(
            &mut state,
            crate::block::Block {
                hash: alt_hash,
                ..child_a.clone()
            },
        );

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: child_a.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Prepare,
            })
            .unwrap();

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: alt_hash,
                height: 1,
                round: 1,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::EquivocatingVote.to_string()
        );
    }

    #[test]
    fn admits_equal_height_sibling_and_keeps_deterministic_head() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);

        let child_low =
            BlockBuilder::build(1, genesis.hash, 1, 0, 1, 2, [1u8; 32], BlockBody::default())
                .unwrap();
        let child_high =
            BlockBuilder::build(1, genesis.hash, 1, 0, 2, 3, [1u8; 32], BlockBody::default())
                .unwrap();

        state.admit_block(child_low.clone()).unwrap();
        state.admit_block(child_high.clone()).unwrap();

        assert_eq!(
            state.fork_choice.get_head(),
            Some(child_low.hash.max(child_high.hash)),
            "fork-choice head must remain deterministic for equal-height siblings",
        );
    }

    #[test]
    fn conflicting_prepare_and_commit_are_tracked_independently_by_kind() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 2,
                kind: VoteKind::Prepare,
            })
            .unwrap();
        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 2,
                kind: VoteKind::Commit,
            })
            .unwrap();
    }

    #[test]
    fn rejects_non_genesis_block_with_zero_parent() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = make_block([0u8; 32], 2, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidGenesisParent.to_string()
        );
    }

    #[test]
    fn rejects_block_with_unknown_parent() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = make_block([7u8; 32], 1, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(err.to_string(), ConsensusError::UnknownParent.to_string());
    }

    #[test]
    fn rejects_block_with_invalid_parent_height() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let block = make_block(genesis.hash, 2, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidParentHeight.to_string()
        );
    }

    #[test]
    fn rejects_duplicate_block_hash() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);

        let err = state.admit_block(genesis).unwrap_err();
        assert_eq!(err.to_string(), ConsensusError::DuplicateBlock.to_string());
    }

    #[test]
    fn accepts_valid_child_with_exact_height_continuity() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let child = make_block(genesis.hash, 1, [1u8; 32]);

        state.admit_block(child.clone()).unwrap();
        assert!(state.fork_choice.contains(child.hash));
    }

    #[test]
    fn rejects_block_with_tampered_hash() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);

        let mut block = make_block([0u8; 32], 0, [1u8; 32]);
        block.hash = [99u8; 32];

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidBlockHash.to_string()
        );
    }

    #[test]
    fn rejects_block_with_tampered_body_commitments() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);

        let mut block = make_block([0u8; 32], 0, [1u8; 32]);
        block.header.body_root = [77u8; 32];
        block.hash = crate::block::hash::compute_block_hash(&block.header);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidBlockBodyCommitments.to_string()
        );
    }

    #[test]
    fn finalization_builds_deterministic_quorum_certificate() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        let seal = state.try_finalize(block.hash, 0).unwrap();
        assert_eq!(seal.attestation_root, seal.certificate.certificate_hash);
        assert_eq!(seal.certificate.signers, vec![[1u8; 32]]);
        assert_eq!(seal.certificate.height, 0);
        assert_eq!(state.fork_choice.finalized_head(), Some(block.hash));
    }

    #[test]
    fn finalization_rejects_commit_votes_from_wrong_round() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert!(state.try_finalize(block.hash, 0).is_none());
    }

    #[test]
    fn finalized_branch_prunes_conflicting_blocks_and_votes() {
        let mut state = state_with_validators(vec![
            validator(1, 1, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let canonical = make_block(genesis.hash, 1, [1u8; 32]);
        let conflicting = crate::block::Block {
            hash: [42u8; 32],
            ..make_block(genesis.hash, 1, [2u8; 32])
        };

        state.admit_block(canonical.clone()).unwrap();
        inject_known_block(&mut state, conflicting.clone());

        for voter in [[1u8; 32], [2u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: canonical.hash,
                    height: 1,
                    round: 1,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        state
            .vote_pool
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: conflicting.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap();

        state.try_finalize(canonical.hash, 1).unwrap();

        assert!(state.blocks.contains_key(&canonical.hash));
        assert!(!state.blocks.contains_key(&conflicting.hash));
        assert_eq!(
            state
                .vote_pool
                .count_for_block_kind(conflicting.hash, VoteKind::Commit),
            0
        );
    }

    #[test]
    fn rejects_stale_votes_for_non_finalized_branch_after_finalization() {
        let mut state = state_with_validators(vec![
            validator(1, 1, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let canonical = make_block(genesis.hash, 1, [1u8; 32]);
        let child = make_block(canonical.hash, 2, [2u8; 32]);
        let conflicting = crate::block::Block {
            hash: [77u8; 32],
            ..make_block(genesis.hash, 1, [3u8; 32])
        };

        state.admit_block(canonical.clone()).unwrap();
        state.admit_block(child.clone()).unwrap();
        inject_known_block(&mut state, conflicting.clone());

        for voter in [[1u8; 32], [2u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: canonical.hash,
                    height: 1,
                    round: 1,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        state.try_finalize(canonical.hash, 1).unwrap();

        let err = state
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: conflicting.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::VoteForUnknownBlock.to_string()
        );

        state
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: child.hash,
                height: 2,
                round: 2,
                kind: VoteKind::Prepare,
            })
            .unwrap();
    }

    #[test]
    fn weighted_quorum_uses_power_not_validator_count() {
        let rotation = ValidatorRotation::new(vec![
            validator(1, 8, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ])
        .unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert!(state.has_quorum(block.hash, VoteKind::Commit));
        assert!(state.try_finalize(block.hash, 0).is_some());
    }

    #[test]
    fn insufficient_weight_rejects_count_majority_finalization() {
        let rotation = ValidatorRotation::new(vec![
            validator(1, 6, ValidatorRole::Validator, true),
            validator(2, 2, ValidatorRole::Validator, true),
            validator(3, 2, ValidatorRole::Validator, true),
        ])
        .unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = admit_genesis(&mut state, [1u8; 32]);

        for voter in [[2u8; 32], [3u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: block.hash,
                    height: 0,
                    round: 0,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        assert!(!state.has_quorum(block.hash, VoteKind::Commit));
        assert!(state.try_finalize(block.hash, 0).is_none());
    }

    #[test]
    fn add_signed_vote_accepts_valid_signature_end_to_end() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let voter = signing_key.verifying_key().to_bytes();
        let mut validator = Validator::new(voter, 10, ValidatorRole::Validator);
        validator.active = true;
        let mut state = state_with_validators(vec![validator]);
        let block = admit_genesis(&mut state, voter);
        let vote = Vote {
            voter,
            block_hash: block.hash,
            height: 0,
            round: 0,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();

        state
            .add_signed_vote(crate::vote::SignedVote { vote, signature })
            .unwrap();

        assert_eq!(
            state
                .vote_pool
                .count_for_block_kind(block.hash, VoteKind::Commit),
            1
        );
    }

    #[test]
    fn add_signed_vote_surfaces_consensus_admission_failure() {
        let signing_key = SigningKey::from_bytes(&[11u8; 32]);
        let voter = signing_key.verifying_key().to_bytes();
        let mut validator = Validator::new(voter, 10, ValidatorRole::Validator);
        validator.active = true;
        let mut state = state_with_validators(vec![validator]);
        let _genesis = admit_genesis(&mut state, voter);

        let vote = Vote {
            voter,
            block_hash: [9u8; 32],
            height: 0,
            round: 0,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();

        let err = state
            .add_signed_vote(crate::vote::SignedVote { vote, signature })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            crate::vote::VoteAuthenticationError::ConsensusAdmissionRejected(
                ConsensusError::VoteForUnknownBlock.to_string()
            )
            .to_string()
        );
    }

    #[test]
    fn add_signed_vote_rejects_invalid_signature_end_to_end() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let voter = signing_key.verifying_key().to_bytes();
        let mut validator = Validator::new(voter, 10, ValidatorRole::Validator);
        validator.active = true;
        let mut state = state_with_validators(vec![validator]);
        let block = admit_genesis(&mut state, voter);
        let mut vote = Vote {
            voter,
            block_hash: block.hash,
            height: 0,
            round: 0,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();
        vote.round = 1;

        let err = state
            .add_signed_vote(crate::vote::SignedVote { vote, signature })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            crate::vote::VoteAuthenticationError::InvalidSignature.to_string()
        );
    }

    #[test]
    fn add_signed_vote_rejects_malformed_key_end_to_end() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let malformed_voter = (0u8..=u8::MAX)
            .map(|byte| [byte; 32])
            .find(|candidate| VerifyingKey::from_bytes(candidate).is_err())
            .expect("at least one malformed 32-byte encoding should be rejected");

        let err = state
            .add_signed_vote(crate::vote::SignedVote {
                vote: Vote {
                    voter: malformed_voter,
                    block_hash: block.hash,
                    height: 0,
                    round: 0,
                    kind: VoteKind::Commit,
                },
                signature: vec![0u8; 64],
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            crate::vote::VoteAuthenticationError::MalformedPublicKey.to_string()
        );
    }

    #[test]
    fn authenticated_vote_rejects_context_mismatch() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let err = state
            .add_authenticated_vote(
                VerifiedAuthenticatedVote {
                    vote: Vote {
                        voter: [1u8; 32],
                        block_hash: block.hash,
                        height: 0,
                        round: 0,
                        kind: VoteKind::Commit,
                    },
                    context: VoteAuthenticationContext {
                        network_id: 2626,
                        epoch: 7,
                        validator_set_root: [9u8; 32],
                        pq_attestation_root: [9u8; 32],
                        signature_scheme: 1,
                    },
                },
                VoteAuthenticationContext {
                    network_id: 2626,
                    epoch: 0,
                    validator_set_root: state.rotation.validator_set_hash(),
                    pq_attestation_root: state.rotation.pq_attestation_root(),
                    signature_scheme: 1,
                },
            )
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidAuthenticatedContext.to_string()
        );
    }

    #[test]
    fn try_finalize_is_idempotent_after_first_success() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        let first = state.try_finalize(block.hash, 0);
        let second = state.try_finalize(block.hash, 0);

        assert!(first.is_some());
        assert!(second.is_some());
        assert_eq!(first, second);
    }
}
