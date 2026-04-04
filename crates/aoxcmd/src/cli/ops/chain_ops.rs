use super::*;

pub fn cmd_chain_status(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct ChainStatus {
        network_id: u32,
        current_height: u64,
        latest_block_hash: String,
        latest_parent_hash: String,
        latest_timestamp_unix: u64,
        produced_blocks: u64,
        running: bool,
        profile: String,
        consensus_mode: &'static str,
    }

    let settings = effective_settings_for_ops()?;
    let state = lifecycle::load_state()?;
    let status = ChainStatus {
        network_id: state.consensus.network_id,
        current_height: state.current_height,
        latest_block_hash: state.consensus.last_block_hash_hex,
        latest_parent_hash: state.consensus.last_parent_hash_hex,
        latest_timestamp_unix: state.consensus.last_timestamp_unix,
        produced_blocks: state.produced_blocks,
        running: state.running,
        profile: settings.profile,
        consensus_mode: "aoxcunity",
    };

    emit_serialized(&status, output_format(args))
}

pub fn cmd_block_get(args: &[String]) -> Result<(), AppError> {
    #[derive(serde::Serialize)]
    struct BlockView {
        requested_height: Option<String>,
        requested_hash: Option<String>,
        available: bool,
        height: u64,
        block_hash: String,
        parent_hash: String,
        proposer: String,
        consensus_round: u64,
        timestamp_unix: u64,
        section_count: usize,
        tx_count: usize,
        tx_hashes: Vec<String>,
        state_root: String,
    }

    let requested_height = arg_value(args, "--height").and_then(|v| normalize_text(&v, false));
    let requested_hash = arg_value(args, "--hash").and_then(|v| normalize_text(&v, false));
    if requested_height.is_some() && requested_hash.is_some() {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Use either --height or --hash, not both",
        ));
    }
    let state = lifecycle::load_state()?;
    let canonical_height = state.current_height;
    let state_root = derive_state_root(&state)?;
    let default_height = "latest".to_string();
    let requested_height_value = requested_height
        .as_deref()
        .unwrap_or(default_height.as_str());

    let historical = load_historical_block(requested_height_value, requested_hash.as_deref())?;
    let (available, height, block_hash, parent_hash, tx_hashes) = if let Some(envelope) = historical
    {
        let tx_hashes = historical_tx_hashes(&envelope);
        (
            true,
            envelope.height,
            envelope.block_hash_hex,
            envelope.parent_hash_hex,
            tx_hashes,
        )
    } else {
        let available = match requested_height_value {
            "latest" => true,
            value => value.parse::<u64>().ok() == Some(canonical_height),
        } && match requested_hash.as_ref() {
            Some(hash) => hash.eq_ignore_ascii_case(&state.consensus.last_block_hash_hex),
            None => true,
        };
        let tx_hashes = if state.last_tx == "none" {
            Vec::new()
        } else {
            vec![state.last_tx.clone()]
        };
        (
            available,
            canonical_height,
            state.consensus.last_block_hash_hex.clone(),
            state.consensus.last_parent_hash_hex.clone(),
            tx_hashes,
        )
    };

    let view = BlockView {
        requested_height: Some(requested_height_value.to_string()),
        requested_hash,
        available,
        height,
        block_hash,
        parent_hash,
        proposer: state.consensus.last_proposer_hex,
        consensus_round: state.consensus.last_round,
        timestamp_unix: state.consensus.last_timestamp_unix,
        section_count: state.consensus.last_section_count,
        tx_count: tx_hashes.len(),
        tx_hashes,
        state_root,
    };

    emit_serialized(&view, output_format(args))
}
