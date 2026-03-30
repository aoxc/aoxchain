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
fn hack_test_rejects_parallel_sibling_fork_injection() -> Result<(), ConsensusError> {
    let mut state = state_with_validators()?;
    let genesis = build_block([0; 32], 1, 0, [7; 32], 8)?;
    let canonical_child = build_block(genesis.hash, 2, 1, [1; 32], 9)?;
    let conflicting_sibling = build_block(genesis.hash, 2, 1, [2; 32], 10)?;
    assert!(state.admit_block(genesis).is_ok());
    assert!(state.admit_block(canonical_child).is_ok());
    assert!(matches!(
        state.admit_block(conflicting_sibling),
        Err(ConsensusError::HeightRegression)
    ));
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
