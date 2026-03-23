use aoxcunity::{
    Block, BlockBody, BlockBuilder, ConsensusEngine, ConsensusEvent, ConsensusState, KernelEffect,
    QuorumThreshold, Validator, ValidatorRole, ValidatorRotation, VerifiedAuthenticatedVote, Vote,
    VoteAuthenticationContext, VoteKind,
};

const ROUNDS: u64 = 24;

fn validator(id: u8, power: u64) -> Validator {
    Validator::new([id; 32], power, ValidatorRole::Validator)
}

fn engine() -> ConsensusEngine {
    let rotation = ValidatorRotation::new(vec![
        validator(1, 5),
        validator(2, 4),
        validator(3, 3),
        validator(4, 2),
    ])
    .expect("validator set should be valid");
    ConsensusEngine::new(ConsensusState::new(rotation, QuorumThreshold::two_thirds()))
}

fn context(engine: &ConsensusEngine, epoch: u64) -> VoteAuthenticationContext {
    VoteAuthenticationContext {
        network_id: 2626,
        epoch,
        validator_set_root: engine.state.rotation.validator_set_hash(),
        signature_scheme: 1,
    }
}

fn block(parent_hash: [u8; 32], height: u64, round: u64, proposer: [u8; 32]) -> Block {
    BlockBuilder::build(
        2626,
        parent_hash,
        height,
        0,
        round,
        1_735_689_600 + height + round,
        proposer,
        BlockBody::default(),
    )
    .expect("block should build")
}

fn commit_event(engine: &ConsensusEngine, voter: u8, block: &Block, epoch: u64) -> ConsensusEvent {
    ConsensusEvent::AdmitVerifiedVote(aoxcunity::VerifiedVote {
        authenticated_vote: VerifiedAuthenticatedVote {
            vote: Vote {
                voter: [voter; 32],
                block_hash: block.hash,
                height: block.header.height,
                round: block.header.round,
                kind: VoteKind::Commit,
            },
            context: context(engine, epoch),
        },
        verification_tag: [voter; 32],
    })
}

#[test]
fn soak_kernel_finalizes_a_long_canonical_chain_deterministically() {
    let mut left = engine();
    let mut right = engine();
    let mut parent_hash = [0u8; 32];
    let mut finalized = Vec::new();

    for height in 0..ROUNDS {
        let round = height;
        let proposer = [((height % 4) + 1) as u8; 32];
        let block = block(parent_hash, height, round, proposer);

        for engine in [&mut left, &mut right] {
            let admitted = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
            assert_eq!(
                admitted.accepted_effects,
                vec![KernelEffect::BlockAccepted(block.hash)],
                "block admission diverged at height {height}"
            );
        }

        for voter in [1u8, 2u8, 3u8] {
            let left_vote = left.apply_event(commit_event(&left, voter, &block, 0));
            let right_vote = right.apply_event(commit_event(&right, voter, &block, 0));
            assert_eq!(
                left_vote, right_vote,
                "vote admission diverged at height {height}"
            );
        }

        let left_finality = left.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        let right_finality = right.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        assert_eq!(
            left_finality, right_finality,
            "finality diverged at height {height}"
        );
        assert!(
            left_finality
                .accepted_effects
                .contains(&KernelEffect::BlockFinalized(block.hash)),
            "expected finalized block effect at height {height}"
        );

        finalized.push(block.hash);
        parent_hash = block.hash;
    }

    assert_eq!(left.state.fork_choice.finalized_head(), Some(parent_hash));
    assert_eq!(right.state.fork_choice.finalized_head(), Some(parent_hash));
    assert_eq!(left.current_height, ROUNDS - 1);
    assert_eq!(left.state.round.round, right.state.round.round);
    assert_eq!(left.lock_state, right.lock_state);
    assert_eq!(left.evidence_buffer, right.evidence_buffer);
    assert_eq!(finalized.len() as u64, ROUNDS);
}
