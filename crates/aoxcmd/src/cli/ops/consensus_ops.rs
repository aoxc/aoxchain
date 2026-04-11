use super::*;

fn parse_optional_u64_flag(
    args: &[String],
    flag: &str,
    context: &str,
) -> Result<Option<u64>, AppError> {
    let Some(value) = parse_optional_text_arg(args, flag, false) else {
        return Ok(None);
    };
    Ok(Some(parse_positive_u64_value(&value, flag, context)?))
}

fn parse_optional_u32_flag(
    args: &[String],
    flag: &str,
    context: &str,
) -> Result<Option<u32>, AppError> {
    let Some(value) = parse_optional_text_arg(args, flag, false) else {
        return Ok(None);
    };
    let parsed = parse_positive_u64_value(&value, flag, context)?;
    u32::try_from(parsed).map(Some).map_err(|_| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Flag {flag} must be in u32 range"),
        )
    })
}

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
    let expected_network_id =
        parse_optional_u32_flag(args, "--expected-network-id", "consensus status")?;
    let min_finalized_height =
        parse_optional_u64_flag(args, "--min-finalized-height", "consensus status")?;
    let strict = has_flag(args, "--strict");

    if strict && !state.running {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Consensus status strict mode requires a running node",
        ));
    }

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

    if let Some(expected) = expected_network_id
        && status.network_id != expected
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus network_id mismatch: expected {expected}, got {}",
                status.network_id
            ),
        ));
    }
    if let Some(min_height) = min_finalized_height
        && status.finalized_height < min_height
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus finalized height {} is below required minimum {min_height}",
                status.finalized_height
            ),
        ));
    }

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

    let state =
        lifecycle::load_state().unwrap_or_else(|_| crate::node::state::NodeState::bootstrap());
    let min_voting_power =
        parse_optional_u64_flag(args, "--min-voting-power", "consensus validators")?;
    let only_active = has_flag(args, "--only-active");
    let target_validator = parse_optional_text_arg(args, "--validator-id", true);

    let validators = vec![ValidatorView {
        validator_id: state.consensus.last_proposer_hex.clone(),
        voting_power: 1,
        status: if state.running { "active" } else { "inactive" },
        proposer_priority: 0,
    }]
    .into_iter()
    .filter(|validator| {
        min_voting_power
            .map(|minimum| validator.voting_power >= minimum)
            .unwrap_or(true)
    })
    .filter(|validator| !only_active || validator.status == "active")
    .filter(|validator| {
        target_validator
            .as_ref()
            .map(|target| validator.validator_id.eq_ignore_ascii_case(target))
            .unwrap_or(true)
    })
    .collect::<Vec<_>>();

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
    let requested_height = parse_optional_u64_flag(args, "--height", "consensus proposer")?;
    let expected_proposer = parse_optional_text_arg(args, "--expected-proposer", true);
    let requested_round = parse_optional_u64_flag(args, "--round", "consensus proposer")?;
    let response = ProposerView {
        proposer_id: state.consensus.last_proposer_hex,
        height: requested_height.unwrap_or(state.current_height),
        round: requested_round.unwrap_or(state.consensus.last_round),
        timestamp_unix: state.consensus.last_timestamp_unix,
    };

    if let Some(expected) = expected_proposer
        && !response.proposer_id.eq_ignore_ascii_case(&expected)
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus proposer mismatch: expected {expected}, got {}",
                response.proposer_id
            ),
        ));
    }

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
    let min_round = parse_optional_u64_flag(args, "--min-round", "consensus round")?;
    let expect_quorum = has_flag(args, "--expect-quorum");
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

    if let Some(required) = min_round
        && response.round < required
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus round {} is below required minimum {required}",
                response.round
            ),
        ));
    }
    if expect_quorum && response.quorum_status != "single-node-ok" {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus quorum is not satisfied: {}",
                response.quorum_status
            ),
        ));
    }

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
    let max_pending_height =
        parse_optional_u64_flag(args, "--max-pending-height", "consensus finality")?;
    let enforce_qc = has_flag(args, "--require-quorum-certificate");
    let response = FinalityView {
        head_height: state.current_height,
        safe_height: state.current_height,
        finalized_height: state.current_height,
        pending_height: state.current_height.saturating_add(1),
        mode: "single-node",
    };

    if let Some(max_pending) = max_pending_height
        && response.pending_height > max_pending
    {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!(
                "Consensus pending height {} exceeds allowed maximum {max_pending}",
                response.pending_height
            ),
        ));
    }
    if enforce_qc && !state.running {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Consensus quorum certificate requirement failed: node is not running",
        ));
    }

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
    let min_commits = parse_optional_u64_flag(args, "--min-commits", "consensus commits")?;
    let requested_round = parse_optional_u64_flag(args, "--round", "consensus commits")?;
    let response = CommitView {
        height: state.current_height,
        block_hash: state.consensus.last_block_hash_hex,
        commits: vec![CommitVoteView {
            validator_id: state.consensus.last_proposer_hex,
            vote: "precommit",
            round: requested_round.unwrap_or(state.consensus.last_round),
        }],
    };

    if let Some(required) = min_commits {
        let commits = u64::try_from(response.commits.len()).unwrap_or(0);
        if commits < required {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!(
                    "Consensus commits {} are below required minimum {required}",
                    commits
                ),
            ));
        }
    }

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
    let require_qc = has_flag(args, "--require-qc");
    let require_evidence = parse_optional_text_arg(args, "--require-evidence", true);
    let response = ConsensusEvidence {
        height: state.current_height,
        round: state.consensus.last_round,
        lock_reason: "single-validator-lock",
        quorum_certificate: true,
        evidence: vec!["prevote", "precommit", "commit"],
    };

    if require_qc && !response.quorum_certificate {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Consensus evidence does not contain quorum certificate",
        ));
    }

    if let Some(required) = require_evidence {
        if !response
            .evidence
            .iter()
            .any(|item| item.eq_ignore_ascii_case(&required))
        {
            return Err(AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Consensus evidence item '{required}' not found"),
            ));
        }
    }

    emit_serialized(&response, output_format(args))
}
