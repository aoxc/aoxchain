use super::*;

pub(super) fn build_block_for_tx(
    state: &NodeState,
    tx: &str,
    key_material: &KeyMaterial,
) -> Result<Block, AppError> {
    let height = state.current_height.saturating_add(1);
    let round = state.consensus.last_round.saturating_add(1);
    let timestamp = unix_now();

    let parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;

    let proposer_key = proposer_key_from_material(key_material)?;

    let lane_commitment = LaneCommitment {
        lane_id: 1,
        lane_type: LaneType::Native,
        tx_count: 1,
        input_root: derive_digest32("AOXC-CMD-INPUT", tx.as_bytes()),
        output_root: derive_digest32("AOXC-CMD-OUTPUT", tx.as_bytes()),
        receipt_root: derive_digest32("AOXC-CMD-RECEIPT", tx.as_bytes()),
        state_commitment: derive_digest32("AOXC-CMD-STATE", format!("{height}:{tx}").as_bytes()),
        proof_commitment: derive_digest32("AOXC-CMD-PROOF", format!("{round}:{tx}").as_bytes()),
    };

    let body = BlockBody {
        sections: vec![BlockSection::LaneCommitment(LaneCommitmentSection {
            lanes: vec![lane_commitment],
        })],
    };

    let proposer = Proposer::new(state.consensus.network_id.max(1), proposer_key);

    proposer
        .propose(parent_hash, height, 0, round, timestamp, body)
        .map_err(|error| {
            AppError::with_source(
                ErrorCode::NodeStateInvalid,
                format!("Failed to construct block at height {height}"),
                error,
            )
        })
}

pub(super) fn apply_block_proposal(
    state: &mut NodeState,
    tx: &str,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    apply_block_proposal_with_message(state, tx, block, key_material)
}

pub(super) fn apply_block_proposal_with_message(
    state: &mut NodeState,
    tx: &str,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    validate_block_proposal_against_state(state, block, key_material)?;

    let message = ConsensusMessage::BlockProposal {
        block: block.clone(),
    };

    state.current_height = block.header.height;
    state.produced_blocks = state.produced_blocks.saturating_add(1);
    state.last_tx = tx.to_string();
    state.key_material = snapshot_from_key_material(key_material)?;
    state.consensus = snapshot_from_message_kind(&message, BLOCK_PROPOSAL_MESSAGE_KIND);
    state.touch();

    Ok(())
}

fn validate_block_proposal_against_state(
    state: &NodeState,
    block: &Block,
    key_material: &KeyMaterial,
) -> Result<(), AppError> {
    let expected_height = state.current_height.saturating_add(1);
    if block.header.height != expected_height {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            format!(
                "Block height invariant violated: expected {}, found {}",
                expected_height, block.header.height
            ),
        ));
    }

    let expected_round = state.consensus.last_round.saturating_add(1);
    if block.header.round != expected_round {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            format!(
                "Block round invariant violated: expected {}, found {}",
                expected_round, block.header.round
            ),
        ));
    }

    let expected_parent_hash = decode_hash32(
        &state.consensus.last_block_hash_hex,
        "last_block_hash_hex",
        ErrorCode::NodeStateInvalid,
    )?;
    if block.header.parent_hash != expected_parent_hash {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block parent hash does not match the canonical prior block hash",
        ));
    }

    let expected_proposer = proposer_key_from_material(key_material)?;
    if block.header.proposer != expected_proposer {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block proposer does not match active consensus key material",
        ));
    }

    if block.header.timestamp < MINIMUM_RUNTIME_TIMESTAMP_UNIX {
        return Err(AppError::new(
            ErrorCode::NodeStateInvalid,
            "Block timestamp must be a positive unix timestamp",
        ));
    }

    Ok(())
}

fn snapshot_from_key_material(key_material: &KeyMaterial) -> Result<KeyMaterialSnapshot, AppError> {
    let summary = key_material.summary()?;

    Ok(KeyMaterialSnapshot {
        bundle_fingerprint: summary.bundle_fingerprint,
        operational_state: summary.operational_state,
        consensus_public_key_hex: summary.consensus_public_key,
        transport_public_key_hex: summary.transport_public_key,
    })
}

#[cfg(test)]
pub(super) fn snapshot_from_message(message: &ConsensusMessage) -> ConsensusSnapshot {
    snapshot_from_message_kind(message, BLOCK_PROPOSAL_MESSAGE_KIND)
}

fn snapshot_from_message_kind(
    message: &ConsensusMessage,
    block_proposal_kind: &str,
) -> ConsensusSnapshot {
    match message {
        ConsensusMessage::BlockProposal { block } => ConsensusSnapshot {
            network_id: block.header.network_id,
            last_parent_hash_hex: hex::encode(block.header.parent_hash),
            last_block_hash_hex: hex::encode(block.hash),
            last_proposer_hex: hex::encode(block.header.proposer),
            last_round: block.header.round,
            last_timestamp_unix: block.header.timestamp,
            last_message_kind: block_proposal_kind.to_string(),
            last_section_count: block.body.sections.len(),
        },
        ConsensusMessage::Vote(vote) => ConsensusSnapshot {
            network_id: vote.context.network_id,
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_block_hash_hex: hex::encode(vote.vote.block_hash),
            last_proposer_hex: hex::encode(vote.vote.voter),
            last_round: vote.vote.round,
            last_timestamp_unix: 0,
            last_message_kind: "vote".to_string(),
            last_section_count: 0,
        },
        ConsensusMessage::Finalize { seal, certificate } => ConsensusSnapshot {
            network_id: certificate.network_id,
            last_block_hash_hex: hex::encode(seal.block_hash),
            last_parent_hash_hex: hex::encode([0u8; 32]),
            last_proposer_hex: hex::encode([0u8; 32]),
            last_round: seal.finalized_round,
            last_timestamp_unix: 0,
            last_message_kind: "finalize".to_string(),
            last_section_count: 0,
        },
    }
}
