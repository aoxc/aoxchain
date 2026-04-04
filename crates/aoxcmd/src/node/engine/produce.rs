use super::*;

pub(super) fn produce_once_impl(tx: &str) -> Result<NodeState, AppError> {
    let mut state = load_state()?;
    state.running = true;

    let key_material = crate::keys::loader::load_operator_key()?;
    let block = build_block_for_tx(&state, tx, &key_material)?;
    apply_block_proposal(&mut state, tx, &block, &key_material)?;
    persist_block_envelope(&block)?;

    persist_state(&state)?;
    Ok(state)
}

pub(super) fn run_rounds_impl(rounds: u64, tx_prefix: &str) -> Result<NodeState, AppError> {
    run_rounds_with_observer_impl(rounds, tx_prefix, |_| {})
}

pub(super) fn run_rounds_with_observer_impl<F>(
    rounds: u64,
    tx_prefix: &str,
    mut observer: F,
) -> Result<NodeState, AppError>
where
    F: FnMut(&RoundTelemetry),
{
    if rounds == 0 {
        return Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            "Round count must be greater than zero",
        ));
    }

    let mut state = load_state()?;
    state.running = true;
    let key_material = crate::keys::loader::load_operator_key()?;

    for index in 0..rounds {
        let tx = format!("{tx_prefix}-{index}");
        let block = build_block_for_tx(&state, &tx, &key_material)?;
        apply_block_proposal(&mut state, &tx, &block, &key_material)?;
        persist_block_envelope(&block)?;

        let telemetry = RoundTelemetry {
            round_index: index + 1,
            tx_id: tx,
            height: state.current_height,
            produced_blocks: state.produced_blocks,
            consensus_round: state.consensus.last_round,
            section_count: state.consensus.last_section_count,
            block_hash_hex: state.consensus.last_block_hash_hex.clone(),
            parent_hash_hex: state.consensus.last_parent_hash_hex.clone(),
            timestamp_unix: state.consensus.last_timestamp_unix,
        };
        observer(&telemetry);
    }

    persist_state(&state)?;
    Ok(state)
}

fn persist_block_envelope(block: &Block) -> Result<(), AppError> {
    let db_root = runtime_db_root()?;
    let store = HybridDataStore::new(&db_root, IndexBackend::Redb).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open block index store at {}", db_root.display()),
            error,
        )
    })?;

    let payload = serde_json::to_vec(block).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize block payload for historical storage",
            error,
        )
    })?;

    let parent_hash_hex = hex::encode(block.header.parent_hash);
    let block_hash_hex =
        canonical_envelope_hash_hex(block.header.height, &parent_hash_hex, &payload)?;

    let envelope = BlockEnvelope {
        height: block.header.height,
        block_hash_hex,
        parent_hash_hex,
        payload,
    };

    store.put_block(&envelope).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to persist historical block at height {}",
                envelope.height
            ),
            error,
        )
    })?;

    Ok(())
}

fn canonical_envelope_hash_hex(
    height: u64,
    parent_hash_hex: &str,
    payload: &[u8],
) -> Result<String, AppError> {
    let parent_hash = hex::decode(parent_hash_hex).map_err(|error| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            "Failed to decode parent hash for envelope integrity digest",
            error,
        )
    })?;

    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_BLOCK_V1");
    hasher.update(height.to_le_bytes());
    hasher.update(parent_hash);
    hasher.update((payload.len() as u64).to_le_bytes());
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

fn runtime_db_root() -> Result<PathBuf, AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    Ok(home.join("runtime").join("db"))
}
