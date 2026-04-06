// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcunity::{
    Block, BlockBody, BlockBuilder, BlockSection, ConsensusError, ConsensusState, LaneCommitment,
    LaneCommitmentSection, LaneType, QuorumThreshold, Validator, ValidatorRole, ValidatorRotation,
    Vote, VoteKind,
};
use rand::{RngExt, SeedableRng, rngs::StdRng};
use std::collections::BTreeSet;

fn validator(id: u8) -> Validator {
    Validator::new([id; 32], 1, ValidatorRole::Validator)
}

fn observer(id: u8) -> Validator {
    Validator::new([id; 32], 1, ValidatorRole::Observer)
}

fn build_block(
    parent_hash: [u8; 32],
    height: u64,
    round: u64,
    proposer: [u8; 32],
    lane_seed: u8,
) -> Result<Block, ConsensusError> {
    Ok(BlockBuilder::build(
        11,
        parent_hash,
        height,
        0,
        round,
        1_735_689_600_u64
            .saturating_add(height)
            .saturating_add(round),
        proposer,
        BlockBody {
            sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
                lanes: vec![LaneCommitment {
                    lane_id: height as u32,
                    lane_type: LaneType::Native,
                    tx_count: 1,
                    input_root: [lane_seed; 32],
                    output_root: [lane_seed.saturating_add(1); 32],
                    receipt_root: [lane_seed.saturating_add(2); 32],
                    state_commitment: [lane_seed.saturating_add(3); 32],
                    proof_commitment: [lane_seed.saturating_add(4); 32],
                }],
            })],
        },
    )?)
}

fn vote(voter: [u8; 32], block: &Block, kind: VoteKind) -> Vote {
    Vote {
        voter,
        block_hash: block.hash,
        height: block.header.height,
        round: block.header.round,
        kind,
    }
}

fn state_with_validators() -> Result<ConsensusState, ConsensusError> {
    let rotation =
        ValidatorRotation::new(vec![validator(1), validator(2), validator(3), validator(4)])?;
    Ok(ConsensusState::new(rotation, QuorumThreshold::two_thirds()))
}

#[test]
fn hack_test_rejects_unknown_parent_injection() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let forged = build_block([9; 32], 5, 0, [7; 32], 1)?;
    assert!(matches!(
        state.admit_block(forged),
        Err(ConsensusError::UnknownParent)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_zero_parent_non_genesis_block() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let forged = build_block([0; 32], 2, 0, [7; 32], 2)?;
    assert!(matches!(
        state.admit_block(forged),
        Err(ConsensusError::InvalidGenesisParent)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_invalid_parent_height_gap() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 3)?;
    let child = build_block(genesis.hash, 3, 1, [8; 32], 4)?;
    assert!(state.admit_block(genesis).is_ok());
    assert!(matches!(
        state.admit_block(child),
        Err(ConsensusError::InvalidParentHeight)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_unknown_validator_vote_spoofing() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 5)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    assert!(matches!(
        state.add_vote(vote([9; 32], &genesis, VoteKind::Commit)),
        Err(ConsensusError::ValidatorNotFound)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_non_voting_observer_ballot() -> Result<(), ConsensusError> {
    let rotation =
        ValidatorRotation::new(vec![validator(1), validator(2), validator(3), observer(4)])?;
    let mut state = ConsensusState::new(rotation, QuorumThreshold::two_thirds());
    let genesis = build_block([0; 32], 1, 0, [7; 32], 6)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    assert!(matches!(
        state.add_vote(vote([4; 32], &genesis, VoteKind::Commit)),
        Err(ConsensusError::NonVotingValidator)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_duplicate_vote_replay() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 7)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    let first = vote([1; 32], &genesis, VoteKind::Commit);
    assert!(state.add_vote(first.clone()).is_ok());
    assert!(matches!(
        state.add_vote(first),
        Err(ConsensusError::DuplicateVote)
    ));
    Ok(())
}

#[test]
fn hack_test_accepts_parallel_sibling_forks_with_deterministic_head() -> Result<(), ConsensusError>
{
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 8)?;
    let canonical_child = build_block(genesis.hash, 2, 1, [1; 32], 9)?;
    let conflicting_sibling = build_block(genesis.hash, 2, 1, [2; 32], 10)?;
    assert!(state.admit_block(genesis).is_ok());
    assert!(state.admit_block(canonical_child.clone()).is_ok());
    assert!(state.admit_block(conflicting_sibling.clone()).is_ok());
    assert_eq!(
        state.fork_choice.get_head(),
        Some(canonical_child.hash.max(conflicting_sibling.hash))
    );
    Ok(())
}

#[test]
fn hack_test_rejects_stale_vote_after_finalization() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 11)?;
    let child = build_block(genesis.hash, 2, 1, [1; 32], 12)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    assert!(state.admit_block(child.clone()).is_ok());
    for voter in [[1; 32], [2; 32], [3; 32]] {
        assert!(
            state
                .add_vote(vote(voter, &child, VoteKind::Commit))
                .is_ok()
        );
    }
    assert!(state.try_finalize(child.hash, child.header.round).is_some());
    assert!(matches!(
        state.add_vote(vote([4; 32], &genesis, VoteKind::Commit)),
        Err(ConsensusError::VoteForUnknownBlock)
    ));
    Ok(())
}

#[test]
fn hack_test_prevents_conflicting_branch_after_finality() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 13)?;
    let canonical = build_block(genesis.hash, 2, 1, [1; 32], 14)?;
    let fork = build_block(genesis.hash, 2, 1, [2; 32], 15)?;
    assert!(state.admit_block(genesis).is_ok());
    assert!(state.admit_block(canonical.clone()).is_ok());
    for voter in [[1; 32], [2; 32], [3; 32]] {
        assert!(
            state
                .add_vote(vote(voter, &canonical, VoteKind::Commit))
                .is_ok()
        );
    }
    assert!(
        state
            .try_finalize(canonical.hash, canonical.header.round)
            .is_some()
    );
    assert!(matches!(
        state.admit_block(fork),
        Err(ConsensusError::HeightRegression)
    ));
    Ok(())
}

#[test]
fn hack_test_requires_quorum_before_finalization() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 16)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    assert!(
        state
            .add_vote(vote([1; 32], &genesis, VoteKind::Commit))
            .is_ok()
    );
    assert!(
        state
            .add_vote(vote([2; 32], &genesis, VoteKind::Commit))
            .is_ok()
    );
    assert!(
        state
            .try_finalize(genesis.hash, genesis.header.round)
            .is_none()
    );
    assert!(
        state
            .add_vote(vote([3; 32], &genesis, VoteKind::Commit))
            .is_ok()
    );
    assert!(
        state
            .try_finalize(genesis.hash, genesis.header.round)
            .is_some()
    );
    Ok(())
}

#[test]
fn property_commit_quorum_matches_threshold() -> Result<(), ConsensusError> {
    let mut rng = StdRng::seed_from_u64(0xA0C2_5151);
    for _ in 0..256 {
        let mut state = state_with_validators()?;
        let genesis = build_block([0; 32], 1, 0, [7; 32], 21)?;
        assert!(state.admit_block(genesis.clone()).is_ok());

        let vote_count = rng.random_range(0..8);
        let mut unique = BTreeSet::new();
        for _ in 0..vote_count {
            let voter = rng.random_range(1u8..=4u8);
            unique.insert(voter);
            let _ = state.add_vote(vote([voter; 32], &genesis, VoteKind::Commit));
        }

        let expected = unique.len() >= 3;
        assert_eq!(state.has_quorum(genesis.hash, VoteKind::Commit), expected);
    }
    Ok(())
}

#[test]
fn property_prepare_votes_never_finalize() -> Result<(), ConsensusError> {
    let mut rng = StdRng::seed_from_u64(0xA0C2_7171);
    for _ in 0..256 {
        let mut state = state_with_validators()?;
        let genesis = build_block([0; 32], 1, 0, [7; 32], 22)?;
        assert!(state.admit_block(genesis.clone()).is_ok());

        let vote_count = rng.random_range(0..8);
        for _ in 0..vote_count {
            let voter = rng.random_range(1u8..=4u8);
            let _ = state.add_vote(vote([voter; 32], &genesis, VoteKind::Prepare));
        }

        assert!(
            state
                .try_finalize(genesis.hash, genesis.header.round)
                .is_none()
        );
    }
    Ok(())
}

#[test]
fn hack_test_rejects_equivocating_vote_same_round() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 23)?;
    let branch_a = build_block(genesis.hash, 2, 5, [1; 32], 24)?;
    let branch_b = build_block(genesis.hash, 2, 5, [2; 32], 25)?;

    assert!(state.admit_block(genesis).is_ok());
    assert!(state.admit_block(branch_a.clone()).is_ok());
    assert!(state.admit_block(branch_b.clone()).is_ok());
    assert!(
        state
            .add_vote(vote([1; 32], &branch_a, VoteKind::Commit))
            .is_ok()
    );
    assert!(matches!(
        state.add_vote(vote([1; 32], &branch_b, VoteKind::Commit)),
        Err(ConsensusError::EquivocatingVote)
    ));
    Ok(())
}

#[test]
fn hack_test_rejects_vote_for_conflicting_branch_after_finality() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 26)?;
    let canonical = build_block(genesis.hash, 2, 1, [1; 32], 27)?;
    let conflicting = build_block(genesis.hash, 2, 1, [2; 32], 28)?;

    assert!(state.admit_block(genesis).is_ok());
    assert!(state.admit_block(canonical.clone()).is_ok());
    assert!(state.admit_block(conflicting.clone()).is_ok());

    for voter in [[1; 32], [2; 32], [3; 32]] {
        assert!(
            state
                .add_vote(vote(voter, &canonical, VoteKind::Commit))
                .is_ok()
        );
    }
    assert!(
        state
            .try_finalize(canonical.hash, canonical.header.round)
            .is_some()
    );

    assert!(matches!(
        state.add_vote(vote([4; 32], &conflicting, VoteKind::Commit)),
        Err(ConsensusError::StaleVote | ConsensusError::VoteForUnknownBlock)
    ));
    Ok(())
}

#[test]
fn hack_test_stake_reduction_can_drop_quorum() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 29)?;
    assert!(state.admit_block(genesis.clone()).is_ok());

    for voter in [[1; 32], [2; 32], [3; 32]] {
        assert!(
            state
                .add_vote(vote(voter, &genesis, VoteKind::Commit))
                .is_ok()
        );
    }
    assert!(state.has_quorum(genesis.hash, VoteKind::Commit));

    let _slashed = state.slash_validator([1; 32], 1, 1, aoxcunity::SlashFault::Equivocation)?;
    let _unbonded = state.unbond_validator([2; 32], 1)?;
    let _undelegated = state.undelegate_from_validator([3; 32], 1)?;

    assert!(!state.has_quorum(genesis.hash, VoteKind::Commit));
    Ok(())
}

#[test]
fn property_finalize_requires_round_matched_commit_votes() -> Result<(), ConsensusError> {
    let mut rng = StdRng::seed_from_u64(0xA0C2_9090);
    for _ in 0..256 {
        let mut state = state_with_validators()?;
        let genesis = build_block([0; 32], 1, 0, [7; 32], 30)?;
        let candidate_round = rng.random_range(1..=8);
        let candidate = build_block(genesis.hash, 2, candidate_round, [1; 32], 31)?;

        assert!(state.admit_block(genesis).is_ok());
        assert!(state.admit_block(candidate.clone()).is_ok());

        for voter in [[1; 32], [2; 32], [3; 32]] {
            let mut forged = vote(voter, &candidate, VoteKind::Commit);
            if rng.random_bool(0.5) {
                forged.round = forged.round.saturating_add(1);
            }
            assert!(state.add_vote(forged).is_ok());
        }

        let finalizable = state.finalizable_round(candidate.hash);
        let finalized = state.try_finalize(candidate.hash, candidate.header.round);

        if finalizable == Some(candidate.header.round) {
            assert!(finalized.is_some());
        } else {
            assert!(finalized.is_none());
        }
    }
    Ok(())
}

#[test]
fn property_randomized_fork_vote_attacks_preserve_safety() -> Result<(), ConsensusError> {
    let mut rng = StdRng::seed_from_u64(0xA0C2_BEEF);
    for _ in 0..128 {
        let mut state = state_with_validators()?;
        let genesis = build_block([0; 32], 1, 0, [7; 32], 40)?;
        let a = build_block(genesis.hash, 2, 1, [1; 32], 41)?;
        let b = build_block(genesis.hash, 2, 1, [2; 32], 42)?;

        assert!(state.admit_block(genesis).is_ok());
        assert!(state.admit_block(a.clone()).is_ok());
        assert!(state.admit_block(b.clone()).is_ok());

        for _ in 0..20 {
            let voter = [rng.random_range(1u8..=4u8); 32];
            let target = if rng.random_bool(0.5) { &a } else { &b };
            let kind = if rng.random_bool(0.5) {
                VoteKind::Prepare
            } else {
                VoteKind::Commit
            };
            let mut candidate = vote(voter, target, kind);
            if rng.random_bool(0.2) {
                candidate.round = candidate.round.saturating_add(1);
            }
            let _ = state.add_vote(candidate);
        }

        let a_finalized = state.try_finalize(a.hash, a.header.round).is_some();
        let b_finalized = state.try_finalize(b.hash, b.header.round).is_some();
        assert!(!(a_finalized && b_finalized));
    }
    Ok(())
}

#[test]
fn hard_test_rejects_tampered_block_integrity_under_forgery_attempts() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 50)?;
    assert!(state.admit_block(genesis.clone()).is_ok());

    let mut hash_forged = build_block(genesis.hash, 2, 1, [1; 32], 51)?;
    hash_forged.hash = [0xAA; 32];
    assert!(matches!(
        state.admit_block(hash_forged),
        Err(ConsensusError::InvalidBlockHash)
    ));

    let mut commitment_forged = build_block(genesis.hash, 2, 2, [2; 32], 52)?;
    if let Some(BlockSection::LaneCommitment(section)) = commitment_forged.body.sections.get_mut(0)
        && let Some(lane) = section.lanes.get_mut(0)
    {
        lane.tx_count = lane.tx_count.saturating_add(1);
    }
    assert!(matches!(
        state.admit_block(commitment_forged),
        Err(ConsensusError::InvalidBlockBodyCommitments)
    ));

    Ok(())
}

#[test]
fn hard_property_long_running_attack_campaign_preserves_finality_safety() -> Result<(), ConsensusError>
{
    let mut state = state_with_validators()?;
    let mut rng = StdRng::seed_from_u64(0xA0C2_DADA);

    let genesis = build_block([0; 32], 1, 0, [7; 32], 60)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    for voter in [[1; 32], [2; 32], [3; 32]] {
        assert!(
            state
                .add_vote(vote(voter, &genesis, VoteKind::Commit))
                .is_ok()
        );
    }
    assert!(state.try_finalize(genesis.hash, genesis.header.round).is_some());

    let mut finalized_hash = genesis.hash;
    let mut finalized_height = genesis.header.height;
    for step in 0..96u8 {
        let height = finalized_height.saturating_add(1);
        let round = (step as u64 % 7).saturating_add(1);
        let branch_a = build_block(
            finalized_hash,
            height,
            round,
            [1; 32],
            61u8.saturating_add(step),
        )?;
        let branch_b = build_block(
            finalized_hash,
            height,
            round,
            [2; 32],
            161u8.saturating_add(step),
        )?;

        assert!(state.admit_block(branch_a.clone()).is_ok());
        assert!(state.admit_block(branch_b.clone()).is_ok());

        let preferred = if rng.random_bool(0.5) {
            branch_a.clone()
        } else {
            branch_b.clone()
        };
        let alternate = if preferred.hash == branch_a.hash {
            branch_b
        } else {
            branch_a
        };

        // Malicious actor attempts equivocation on sibling branches.
        let _ = state.add_vote(vote([4; 32], &preferred, VoteKind::Commit));
        let _ = state.add_vote(vote([4; 32], &alternate, VoteKind::Commit));

        for voter in [[1; 32], [2; 32], [3; 32]] {
            assert!(
                state
                    .add_vote(vote(voter, &preferred, VoteKind::Commit))
                    .is_ok()
            );
        }

        let preferred_seal = state.try_finalize(preferred.hash, preferred.header.round);
        let alternate_seal = state.try_finalize(alternate.hash, alternate.header.round);
        assert!(preferred_seal.is_some());
        assert!(alternate_seal.is_none());

        finalized_hash = preferred.hash;
        finalized_height = preferred.header.height;

        let fc_finalized = state.fork_choice.finalized_head();
        assert_eq!(fc_finalized, Some(finalized_hash));
        assert!(
            state
                .fork_choice
                .get(finalized_hash)
                .is_some_and(|meta| meta.height == finalized_height)
        );
    }

    Ok(())
}

#[test]
fn hard_property_massive_randomized_invalid_block_and_vote_noise() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let mut rng = StdRng::seed_from_u64(0xA0C2_FEED);
    let genesis = build_block([0; 32], 1, 0, [7; 32], 70)?;
    assert!(state.admit_block(genesis.clone()).is_ok());
    let mut parent = genesis;
    let mut finalized_height = 1u64;

    for i in 0..128u64 {
        let valid = build_block(parent.hash, finalized_height + 1, i % 5 + 1, [1; 32], i as u8)?;
        assert!(state.admit_block(valid.clone()).is_ok());

        // Inject malformed blocks with random corruption patterns.
        for _ in 0..5 {
            let mut forged = valid.clone();
            match rng.random_range(0..4) {
                0 => forged.hash = [rng.random(); 32],
                1 => forged.header.parent_hash = [0; 32],
                2 => forged.header.height = forged.header.height.saturating_add(2),
                _ => forged.header.lane_root = [rng.random(); 32],
            }
            assert!(state.admit_block(forged).is_err());
        }

        for voter in [[1; 32], [2; 32], [3; 32]] {
            assert!(
                state
                    .add_vote(vote(voter, &valid, VoteKind::Commit))
                    .is_ok()
            );
        }
        assert!(state.try_finalize(valid.hash, valid.header.round).is_some());

        // Vote noise from unknown/stale/equivocating actors should never break state.
        for _ in 0..16 {
            let random_voter = [rng.random(); 32];
            let _ = state.add_vote(vote(random_voter, &valid, VoteKind::Commit));
            let _ = state.add_vote(vote([4; 32], &valid, VoteKind::Prepare));
            let _ = state.add_vote(vote([4; 32], &valid, VoteKind::Commit));
        }

        parent = valid;
        finalized_height = parent.header.height;
        assert_eq!(
            state.fork_choice.finalized_head(),
            Some(parent.hash),
            "finalized head must remain monotonic under heavy malformed traffic"
        );
    }

    Ok(())
}
