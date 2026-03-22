use aoxcunity::{
    Block, BlockBody, BlockBuilder, BlockSection, ConsensusError, ConsensusState, LaneCommitment,
    LaneCommitmentSection, LaneType, QuorumThreshold, Validator, ValidatorRole, ValidatorRotation,
    Vote, VoteKind,
};

fn validator(id: u8) -> Validator {
    Validator::new([id; 32], 1, ValidatorRole::Validator)
}

fn build_block(
    parent_hash: [u8; 32],
    height: u64,
    round: u64,
    proposer: [u8; 32],
    lane_seed: u8,
) -> Block {
    BlockBuilder::build(
        1,
        parent_hash,
        height,
        0,
        round,
        1_735_689_600 + height + round,
        proposer,
        BlockBody {
            sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
                lanes: vec![LaneCommitment {
                    lane_id: height as u32,
                    lane_type: LaneType::Native,
                    tx_count: 1,
                    input_root: [lane_seed; 32],
                    output_root: [lane_seed.wrapping_add(1); 32],
                    receipt_root: [lane_seed.wrapping_add(2); 32],
                    state_commitment: [lane_seed.wrapping_add(3); 32],
                    proof_commitment: [lane_seed.wrapping_add(4); 32],
                }],
            })],
        },
    )
    .expect("test block should build")
}

fn vote(voter: [u8; 32], block_hash: [u8; 32], height: u64, round: u64, kind: VoteKind) -> Vote {
    Vote {
        voter,
        block_hash,
        height,
        round,
        kind,
    }
}

fn state_with_validators(count: u8) -> ConsensusState {
    let validators = (1..=count).map(validator).collect();
    ConsensusState::new(
        ValidatorRotation::new(validators).expect("validator set should be valid"),
        QuorumThreshold::two_thirds(),
    )
}

#[test]
fn concurrent_nodes_finalize_one_branch_and_reject_a_late_conflicting_fork() {
    let mut node_a = state_with_validators(4);
    let mut node_b = state_with_validators(4);
    let mut observer = state_with_validators(4);

    let genesis = build_block([0; 32], 1, 0, [9; 32], 10);
    for state in [&mut node_a, &mut node_b, &mut observer] {
        state.admit_block(genesis.clone()).unwrap();
    }

    let fork_a = build_block(genesis.hash, 2, 1, [1; 32], 20);
    let fork_b = build_block(genesis.hash, 2, 1, [2; 32], 30);
    node_a.admit_block(fork_a.clone()).unwrap();
    node_b.admit_block(fork_b.clone()).unwrap();
    observer.admit_block(fork_a.clone()).unwrap();

    for validator_id in [[1; 32], [2; 32], [3; 32]] {
        observer
            .add_vote(vote(
                validator_id,
                fork_a.hash,
                fork_a.header.height,
                fork_a.header.round,
                VoteKind::Commit,
            ))
            .unwrap();
    }

    let seal = observer
        .try_finalize(fork_a.hash, fork_a.header.round)
        .expect("fork_a should finalize with 3/4 voting power");
    assert_eq!(seal.block_hash, fork_a.hash);
    assert_eq!(observer.fork_choice.get_head(), Some(fork_a.hash));

    let late_conflict = observer.admit_block(fork_b.clone());
    assert!(matches!(
        late_conflict,
        Err(ConsensusError::HeightRegression)
    ));

    let child_of_finalized = build_block(fork_a.hash, 3, 2, [4; 32], 40);
    observer.admit_block(child_of_finalized.clone()).unwrap();
    assert_eq!(observer.fork_choice.get_head(), Some(fork_a.hash));
    assert_eq!(
        observer
            .fork_choice
            .get(child_of_finalized.hash)
            .unwrap()
            .height,
        3
    );
}

#[test]
fn delayed_and_duplicate_votes_do_not_create_false_quorum() {
    let mut state = state_with_validators(4);

    let block = build_block([0; 32], 1, 0, [8; 32], 50);
    state.admit_block(block.clone()).unwrap();

    state
        .add_vote(vote(
            [1; 32],
            block.hash,
            block.header.height,
            block.header.round,
            VoteKind::Prepare,
        ))
        .unwrap();
    state
        .add_vote(vote(
            [2; 32],
            block.hash,
            block.header.height,
            block.header.round,
            VoteKind::Commit,
        ))
        .unwrap();
    state
        .add_vote(vote(
            [3; 32],
            block.hash,
            block.header.height,
            block.header.round,
            VoteKind::Commit,
        ))
        .unwrap();

    assert!(
        state.try_finalize(block.hash, block.header.round).is_none(),
        "only two commit votes have arrived; quorum must not be reached yet"
    );

    let duplicate = state.add_vote(vote(
        [3; 32],
        block.hash,
        block.header.height,
        block.header.round,
        VoteKind::Commit,
    ));
    assert!(matches!(duplicate, Err(ConsensusError::DuplicateVote)));
    assert_eq!(state.observed_voting_power(block.hash, VoteKind::Commit), 2);

    state
        .add_vote(vote(
            [4; 32],
            block.hash,
            block.header.height,
            block.header.round,
            VoteKind::Commit,
        ))
        .unwrap();

    let seal = state
        .try_finalize(block.hash, block.header.round)
        .expect("quorum should be reached only after the delayed third commit");
    assert_eq!(seal.block_hash, block.hash);
    assert_eq!(state.fork_choice.get_head(), Some(block.hash));
}

#[test]
fn byzantine_equivocation_cannot_finalize_two_competing_forks_without_extra_honest_power() {
    let mut node_a = state_with_validators(4);
    let mut node_b = state_with_validators(4);

    let genesis = build_block([0; 32], 1, 0, [7; 32], 60);
    node_a.admit_block(genesis.clone()).unwrap();
    node_b.admit_block(genesis.clone()).unwrap();

    let fork_a = build_block(genesis.hash, 2, 1, [1; 32], 61);
    let fork_b = build_block(genesis.hash, 2, 1, [2; 32], 62);
    node_a.admit_block(fork_a.clone()).unwrap();
    node_b.admit_block(fork_b.clone()).unwrap();

    node_a
        .add_vote(vote([1; 32], fork_a.hash, 2, 1, VoteKind::Commit))
        .unwrap();
    node_b
        .add_vote(vote([1; 32], fork_b.hash, 2, 1, VoteKind::Commit))
        .unwrap();

    node_a
        .add_vote(vote([2; 32], fork_a.hash, 2, 1, VoteKind::Commit))
        .unwrap();
    node_b
        .add_vote(vote([3; 32], fork_b.hash, 2, 1, VoteKind::Commit))
        .unwrap();

    assert!(node_a.try_finalize(fork_a.hash, 1).is_none());
    assert!(node_b.try_finalize(fork_b.hash, 1).is_none());

    node_a
        .add_vote(vote([4; 32], fork_a.hash, 2, 1, VoteKind::Commit))
        .unwrap();

    let seal_a = node_a
        .try_finalize(fork_a.hash, 1)
        .expect("fork_a should finalize after the delayed honest vote");
    assert_eq!(seal_a.block_hash, fork_a.hash);
    assert!(node_b.try_finalize(fork_b.hash, 1).is_none());
    assert_eq!(node_a.fork_choice.get_head(), Some(fork_a.hash));
    assert_eq!(
        node_b.observed_voting_power(fork_b.hash, VoteKind::Commit),
        2
    );
}
