// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxcunity::{
    Block, BlockBody, BlockBuilder, ConsensusEngine, ConsensusEvent, ConsensusState,
    QuorumThreshold, Validator, ValidatorRole, ValidatorRotation, VerifiedAuthenticatedVote,
    VerifiedTimeoutVote, Vote, VoteAuthenticationContext, VoteKind,
};
use rand::{RngExt, SeedableRng, rngs::StdRng};

const CASES: usize = 128;
const STEPS_PER_CASE: usize = 24;

fn validator(id: u8, power: u64) -> Validator {
    Validator::new([id; 32], power, ValidatorRole::Validator)
}

fn engine() -> ConsensusEngine {
    let rotation = ValidatorRotation::new(vec![
        validator(1, 4),
        validator(2, 3),
        validator(3, 2),
        validator(4, 1),
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

fn block(parent_hash: [u8; 32], height: u64, round: u64, proposer: [u8; 32], seed: u8) -> Block {
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
    .map(|mut block| {
        block.hash[0] ^= seed;
        block
    })
    .expect("block must build")
}

#[test]
fn fuzz_kernel_event_matrix_is_deterministic_and_safe() {
    let mut rng = StdRng::seed_from_u64(0xA0C2_F022);

    for case in 0..CASES {
        let mut left = engine();
        let mut right = engine();
        let genesis = block([0; 32], 0, 0, [9; 32], case as u8);
        let child = block(genesis.hash, 1, 1, [8; 32], case as u8 ^ 0x55);
        let alt = block(genesis.hash, 1, 1, [7; 32], case as u8 ^ 0xAA);
        let known_blocks = [genesis.clone(), child.clone(), alt.clone()];

        let mut events = vec![
            ConsensusEvent::AdmitBlock(genesis.clone()),
            ConsensusEvent::AdmitBlock(child.clone()),
        ];

        for _ in 0..STEPS_PER_CASE {
            match rng.random_range(0..5) {
                0 => {
                    let selected = known_blocks[rng.random_range(0..known_blocks.len())].clone();
                    events.push(ConsensusEvent::AdmitBlock(selected));
                }
                1 | 2 => {
                    let selected = &known_blocks[rng.random_range(0..known_blocks.len())];
                    let epoch = rng.random_range(0..=2);
                    let voter = [rng.random_range(1u8..=4u8); 32];
                    let kind = if rng.random::<bool>() {
                        VoteKind::Commit
                    } else {
                        VoteKind::Prepare
                    };
                    events.push(ConsensusEvent::AdmitVerifiedVote(aoxcunity::VerifiedVote {
                        authenticated_vote: VerifiedAuthenticatedVote {
                            vote: Vote {
                                voter,
                                block_hash: selected.hash,
                                height: selected.header.height,
                                round: selected.header.round,
                                kind,
                            },
                            context: context(&left, epoch),
                        },
                        verification_tag: [case as u8; 32],
                    }));
                }
                3 => {
                    let selected = &known_blocks[rng.random_range(0..known_blocks.len())];
                    events.push(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                        timeout_vote: aoxcunity::TimeoutVote {
                            block_hash: selected.hash,
                            height: selected.header.height,
                            round: selected.header.round,
                            epoch: rng.random_range(0..=2),
                            timeout_round: selected.header.round.saturating_add(1),
                            voter: [rng.random_range(1u8..=4u8); 32],
                        },
                        verification_tag: [0x77; 32],
                    }));
                }
                _ => {
                    let selected = &known_blocks[rng.random_range(0..known_blocks.len())];
                    events.push(ConsensusEvent::EvaluateFinality {
                        block_hash: selected.hash,
                    });
                }
            }
        }

        let left_results: Vec<_> = events
            .iter()
            .cloned()
            .map(|event| left.apply_event(event))
            .collect();
        let right_results: Vec<_> = events
            .into_iter()
            .map(|event| right.apply_event(event))
            .collect();

        assert_eq!(left_results, right_results, "diverged on case {case}");
        assert_eq!(
            left.state.fork_choice.get_head(),
            right.state.fork_choice.get_head()
        );
        assert_eq!(
            left.state.fork_choice.finalized_head(),
            right.state.fork_choice.finalized_head(),
            "finalized head mismatch on case {case}"
        );
        assert_eq!(
            left.lock_state, right.lock_state,
            "lock mismatch on case {case}"
        );
        assert_eq!(
            left.evidence_buffer, right.evidence_buffer,
            "evidence mismatch on case {case}"
        );
    }
}
