use super::*;

pub(in crate::cli::ops) fn derive_state_root(
    state: &crate::node::state::NodeState,
) -> Result<String, AppError> {
    let encoded = serde_json::to_vec(state).map_err(|error| {
        AppError::with_source(
            ErrorCode::OutputEncodingFailed,
            "Failed to serialize node state for state-root derivation",
            error,
        )
    })?;
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    Ok(hex::encode(hasher.finalize()))
}

pub(in crate::cli::ops) fn runtime_db_root() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join("db"))
}

pub(in crate::cli::ops) fn open_runtime_store() -> Result<HybridDataStore, AppError> {
    let db_root = runtime_db_root()?;
    HybridDataStore::new(&db_root, IndexBackend::Redb).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to open runtime data store at {}", db_root.display()),
            error,
        )
    })
}

pub(in crate::cli::ops) fn load_historical_block(
    requested_height: &str,
    requested_hash: Option<&str>,
) -> Result<Option<BlockEnvelope>, AppError> {
    let store = open_runtime_store()?;
    let target_hash = requested_hash.and_then(|value| normalize_text(value, true));

    let requested = normalize_text(requested_height, true).unwrap_or_else(|| "latest".to_string());
    let target_height = if requested == "latest" {
        Some(lifecycle::load_state()?.current_height)
    } else {
        Some(requested.parse::<u64>().map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                "Flag --height must be a positive integer or 'latest'",
            )
        })?)
    };

    let candidate = if let Some(hash) = target_hash.as_deref() {
        match store.get_block_by_hash(hash) {
            Ok(block) => Some(block),
            Err(DataError::NotFound) => None,
            Err(error) => {
                return Err(AppError::with_source(
                    ErrorCode::LedgerInvalid,
                    format!("Failed to load historical block by hash {hash}"),
                    error,
                ));
            }
        }
    } else if let Some(height) = target_height {
        match store.get_block_by_height(height) {
            Ok(block) => Some(block),
            Err(DataError::NotFound) => None,
            Err(error) => {
                return Err(AppError::with_source(
                    ErrorCode::LedgerInvalid,
                    format!("Failed to load historical block at height {height}"),
                    error,
                ));
            }
        }
    } else {
        None
    };

    Ok(candidate.filter(|block| {
        target_height
            .map(|height| block.height == height)
            .unwrap_or(true)
            && target_hash
                .as_ref()
                .map(|hash| block.block_hash_hex.eq_ignore_ascii_case(hash))
                .unwrap_or(true)
    }))
}

pub(in crate::cli::ops) fn historical_tx_hashes(envelope: &BlockEnvelope) -> Vec<String> {
    let payload = match serde_json::from_slice::<Value>(&envelope.payload) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    payload
        .get("body")
        .and_then(|body| body.get("sections"))
        .and_then(Value::as_array)
        .map(|sections| {
            sections
                .iter()
                .filter_map(|section| section.get("payload").and_then(Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(in crate::cli::ops) fn tx_index_path() -> Result<PathBuf, AppError> {
    Ok(resolve_home()?.join("runtime").join(TX_INDEX_FILE))
}

pub(in crate::cli::ops) fn load_tx_index() -> Result<TxIndex, AppError> {
    let path = tx_index_path()?;
    if !path.exists() {
        return Ok(TxIndex {
            entries: BTreeMap::new(),
        });
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read transaction index from {}", path.display()),
            error,
        )
    })?;

    serde_json::from_str::<TxIndex>(&raw).map_err(|error| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!("Failed to parse transaction index from {}", path.display()),
            error,
        )
    })
}

pub(in crate::cli::ops) fn load_tx_index_entry(
    tx_hash: &str,
) -> Result<Option<TxIndexEntry>, AppError> {
    let normalized_hash = tx_hash.to_ascii_lowercase();
    let tx_index = load_tx_index()?;
    Ok(tx_index
        .entries
        .get(&normalized_hash)
        .cloned()
        .or_else(|| tx_index.entries.get(tx_hash).cloned()))
}

pub(in crate::cli::ops) fn tx_hash_hex(tx_payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(tx_payload.as_bytes());
    hex::encode(hasher.finalize())
}

pub(in crate::cli::ops) fn load_state_root_for_height(
    height: u64,
) -> Result<Option<(u64, String)>, AppError> {
    let Some(block) = load_historical_block(&height.to_string(), None)? else {
        return Ok(None);
    };

    let payload = serde_json::from_slice::<Value>(&block.payload).ok();
    let indexed_root = payload
        .as_ref()
        .and_then(|value| value.get("state_root"))
        .and_then(Value::as_str)
        .or_else(|| {
            payload
                .as_ref()
                .and_then(|value| value.get("stateRoot"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            payload
                .as_ref()
                .and_then(|value| value.get("state_root_hex"))
                .and_then(Value::as_str)
        })
        .map(str::to_string)
        .unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(&block.payload);
            hex::encode(hasher.finalize())
        });

    Ok(Some((block.height, indexed_root)))
}
