use super::*;

pub fn cmd_consensus_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusStatus {
        network_id: u32,
        last_round: u64,
        last_message_kind: String,
        latest_block_hash: String,
        latest_parent_hash: String,
        latest_proposer: String,
        latest_timestamp_unix: u64,
        validator_set_hash: String,
        active_validators: u64,
        finalized_height: u64,
        locked_height: u64,
        quorum_status: &'static str,
        produced_blocks: u64,
        current_height: u64,
        updated_at: String,
    }

    let state = lifecycle::load_state()?;
    let status = ConsensusStatus {
        network_id: state.consensus.network_id,
        last_round: state.consensus.last_round,
        last_message_kind: state.consensus.last_message_kind,
        latest_block_hash: state.consensus.last_block_hash_hex,
        latest_parent_hash: state.consensus.last_parent_hash_hex,
        latest_proposer: state.consensus.last_proposer_hex,
        latest_timestamp_unix: state.consensus.last_timestamp_unix,
        validator_set_hash: state.key_material.bundle_fingerprint.clone(),
        active_validators: 1,
        finalized_height: state.current_height,
        locked_height: state.current_height,
        quorum_status: if state.running {
            "single-node-ok"
        } else {
            "idle"
        },
        produced_blocks: state.produced_blocks,
        current_height: state.current_height,
        updated_at: state.updated_at,
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_consensus_validators(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ValidatorView {
        validator_id: String,
        voting_power: u64,
        status: &'static str,
        proposer_priority: i64,
    }

    #[derive(serde::Serialize)]
    struct ValidatorSetView {
        mode: &'static str,
        validator_set_hash: String,
        total_voting_power: u64,
        validators: Vec<ValidatorView>,
    }

    let state = lifecycle::load_state()
        .unwrap_or_else(|_| crate::node::state::NodeState::bootstrap());
    let validators = vec![ValidatorView {
        validator_id: state.consensus.last_proposer_hex.clone(),
        voting_power: 1,
        status: if state.running { "active" } else { "inactive" },
        proposer_priority: 0,
    }];
    let response = ValidatorSetView {
        mode: "single-node",
        validator_set_hash: state.key_material.bundle_fingerprint,
        total_voting_power: 1,
        validators,
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_proposer(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ProposerView {
        proposer_id: String,
        height: u64,
        round: u64,
        timestamp_unix: u64,
    }

    let state = lifecycle::load_state()?;
    let response = ProposerView {
        proposer_id: state.consensus.last_proposer_hex,
        height: state.current_height,
        round: state.consensus.last_round,
        timestamp_unix: state.consensus.last_timestamp_unix,
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_round(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusRoundView {
        height: u64,
        round: u64,
        message_kind: String,
        quorum_status: &'static str,
        timeout_state: &'static str,
    }

    let state = lifecycle::load_state()?;
    let response = ConsensusRoundView {
        height: state.current_height,
        round: state.consensus.last_round,
        message_kind: state.consensus.last_message_kind,
        quorum_status: if state.running {
            "single-node-ok"
        } else {
            "idle"
        },
        timeout_state: "not-triggered",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_finality(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct FinalityView {
        head_height: u64,
        safe_height: u64,
        finalized_height: u64,
        pending_height: u64,
        mode: &'static str,
    }

    let state = lifecycle::load_state()?;
    let response = FinalityView {
        head_height: state.current_height,
        safe_height: state.current_height,
        finalized_height: state.current_height,
        pending_height: state.current_height.saturating_add(1),
        mode: "single-node",
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_commits(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct CommitVoteView {
        validator_id: String,
        vote: &'static str,
        round: u64,
    }

    #[derive(serde::Serialize)]
    struct CommitView {
        height: u64,
        block_hash: String,
        commits: Vec<CommitVoteView>,
    }

    let state = lifecycle::load_state()?;
    let response = CommitView {
        height: state.current_height,
        block_hash: state.consensus.last_block_hash_hex,
        commits: vec![CommitVoteView {
            validator_id: state.consensus.last_proposer_hex,
            vote: "precommit",
            round: state.consensus.last_round,
        }],
    };
    emit_serialized(&response, output_format(args))
}

pub fn cmd_consensus_evidence(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ConsensusEvidence {
        height: u64,
        round: u64,
        lock_reason: &'static str,
        quorum_certificate: bool,
        evidence: Vec<&'static str>,
    }

    let state = lifecycle::load_state()?;
    let response = ConsensusEvidence {
        height: state.current_height,
        round: state.consensus.last_round,
        lock_reason: "single-validator-lock",
        quorum_certificate: true,
        evidence: vec!["prevote", "precommit", "commit"],
    };
    emit_serialized(&response, output_format(args))
}
