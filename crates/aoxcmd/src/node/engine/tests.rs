use super::core::{
    apply_block_proposal_with_message, build_block_for_tx, decode_hash32,
    proposer_key_from_material, run_rounds_with_observer, snapshot_from_message,
};
use crate::{error::ErrorCode, keys::material::KeyMaterial, node::state::NodeState};
use aoxcore::identity::key_bundle::NodeKeyRole;
use aoxcunity::{
    AuthenticatedVote, ConsensusMessage, Vote, VoteAuthenticationContext, VoteKind,
};

#[test]
fn proposer_key_uses_consensus_public_key_from_bundle() {
    let material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let proposer = proposer_key_from_material(&material)
        .expect("active key material should produce proposer key");
    let expected = material
        .bundle
        .keys
        .iter()
        .find(|record| matches!(record.role, NodeKeyRole::Consensus))
        .and_then(|record| hex::decode(&record.public_key).ok())
        .map(|bytes| {
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes[..32]);
            out
        })
        .expect("consensus key must decode");

    assert_eq!(proposer, expected);
}

#[test]
fn build_block_for_tx_advances_height_and_round_from_state_snapshot() {
    let key_material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let mut state = NodeState::bootstrap();
    state.current_height = 4;
    state.consensus.last_round = 8;

    let block =
        build_block_for_tx(&state, "tx-42", &key_material).expect("block build should work");

    assert_eq!(block.header.height, 5);
    assert_eq!(block.header.round, 9);
    assert_eq!(block.body.sections.len(), 1);
}

#[test]
fn apply_block_proposal_updates_runtime_and_consensus_snapshots() {
    let key_material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let mut state = NodeState::bootstrap();
    let block =
        build_block_for_tx(&state, "tx-apply", &key_material).expect("block build should work");

    apply_block_proposal_with_message(&mut state, "tx-apply", &block, &key_material)
        .expect("proposal application should succeed");

    assert_eq!(state.current_height, 1);
    assert_eq!(state.produced_blocks, 1);
    assert_eq!(state.last_tx, "tx-apply");
    assert_eq!(state.consensus.last_message_kind, "block_proposal");
    assert_eq!(state.consensus.last_block_hash_hex, hex::encode(block.hash));
    assert_eq!(state.consensus.last_section_count, 1);
    assert_eq!(
        state.key_material.consensus_public_key_hex.len(),
        64,
        "consensus public key should remain a canonical 32-byte hex string"
    );
}

#[test]
fn apply_block_proposal_rejects_non_sequential_height() {
    let key_material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let mut state = NodeState::bootstrap();
    let mut block = build_block_for_tx(&state, "tx-invalid-height", &key_material)
        .expect("block build should work");
    block.header.height = 9;

    let error = apply_block_proposal_with_message(
        &mut state,
        "tx-invalid-height",
        &block,
        &key_material,
    )
    .expect_err("non-sequential height must fail");

    assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
}

#[test]
fn apply_block_proposal_rejects_parent_hash_mismatch() {
    let key_material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let mut state = NodeState::bootstrap();
    let mut block = build_block_for_tx(&state, "tx-parent", &key_material)
        .expect("block build should work");
    block.header.parent_hash = [9u8; 32];

    let error =
        apply_block_proposal_with_message(&mut state, "tx-parent", &block, &key_material)
            .expect_err("parent hash mismatch must fail");

    assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
}

#[test]
fn decode_hash32_rejects_non_32_byte_payloads() {
    let error = decode_hash32("aa", "test_field", ErrorCode::NodeStateInvalid)
        .expect_err("short decode must fail");

    assert_eq!(error.code(), ErrorCode::NodeStateInvalid.as_str());
}

#[test]
fn run_rounds_with_observer_rejects_zero_rounds() {
    let error =
        run_rounds_with_observer(0, "AOXC-RUN", |_| {}).expect_err("zero rounds must fail");

    assert_eq!(error.code(), ErrorCode::UsageInvalidArguments.as_str());
}

#[test]
fn snapshot_from_message_tracks_vote_payload() {
    let key_material = KeyMaterial::generate("validator-01", "validator", "Test#2026!")
        .expect("key material generation should succeed");
    let state = NodeState::bootstrap();
    let block = build_block_for_tx(&state, "vote-source", &key_material)
        .expect("block build should work");
    let proposer = proposer_key_from_material(&key_material)
        .expect("proposer key should derive from key material");
    let vote = AuthenticatedVote {
        vote: Vote {
            voter: proposer,
            block_hash: block.hash,
            height: block.header.height,
            round: block.header.round,
            kind: VoteKind::Commit,
        },
        context: VoteAuthenticationContext {
            network_id: block.header.network_id,
            epoch: block.header.era,
            validator_set_root: [0u8; 32],
            pq_attestation_root: [0u8; 32],
            signature_scheme: 1,
        },
        signature: vec![0u8; 64],
        pq_public_key: None,
        pq_signature: None,
    };

    let snapshot = snapshot_from_message(&ConsensusMessage::Vote(vote));

    assert_eq!(snapshot.last_message_kind, "vote");
    assert_eq!(snapshot.last_block_hash_hex, hex::encode(block.hash));
    assert_eq!(snapshot.last_proposer_hex, hex::encode(proposer));
    assert_eq!(snapshot.last_round, block.header.round);
}
