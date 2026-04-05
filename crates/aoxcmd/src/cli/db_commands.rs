/// Initializes the effective AOXC database root and emits a normalized status view.
///
/// Operator contract:
/// - The command is safe to invoke repeatedly.
/// - The selected backend is opened under the resolved AOXC root.
/// - A serialized status payload is emitted on success.
pub fn cmd_db_init(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (_, root) = open_store(backend)?;
    let status = build_status(backend, &root)?;
    emit_serialized(&status, output_format(args))
}

/// Emits the current database status for the effective AOXC home.
pub fn cmd_db_status(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (_, root) = open_store(backend)?;
    let status = build_status(backend, &root)?;
    emit_serialized(&status, output_format(args))
}

/// Persists a block envelope loaded from a JSON file.
///
/// Validation and safety flow:
/// 1. Parse backend selection.
/// 2. Require `--block-file`.
/// 3. Read UTF-8 JSON payload from disk.
/// 4. Deserialize into a `BlockEnvelope`.
/// 5. Persist through the selected embedded metadata backend.
///
/// Failure modes are reported with stable AOXC application error categories.
pub fn cmd_db_put_block(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let block_file = parse_required_arg(args, "--block-file")?;

    let encoded = fs::read_to_string(&block_file).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to read block file {block_file}"),
            err,
        )
    })?;

    let block: BlockEnvelope = serde_json::from_str(&encoded).map_err(|err| {
        AppError::with_source(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid block JSON at {block_file}"),
            err,
        )
    })?;

    let (store, root) = open_store(backend)?;
    let meta = store.put_block(&block).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to persist block into {} using {} backend",
                root.display(),
                backend_label(backend)
            ),
            err,
        )
    })?;

    emit_serialized(&meta, output_format(args))
}

/// Loads a block envelope by block height.
pub fn cmd_db_get_height(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let height = parse_u64_arg(args, "--height")?;
    let (store, root) = open_store(backend)?;

    let block = store.get_block_by_height(height).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to load block at height {height} from {} using {} backend",
                root.display(),
                backend_label(backend)
            ),
            err,
        )
    })?;

    emit_serialized(&block, output_format(args))
}

/// Loads a block envelope by block hash.
pub fn cmd_db_get_hash(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let hash = parse_required_arg(args, "--hash")?;
    let (store, root) = open_store(backend)?;

    let block = store.get_block_by_hash(&hash).map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to load block hash {hash} from {} using {} backend",
                root.display(),
                backend_label(backend)
            ),
            err,
        )
    })?;

    emit_serialized(&block, output_format(args))
}

/// Compacts the metadata index and emits the post-compaction database status.
///
/// Operational intent:
/// - Preserve the existing backend selection.
/// - Run compaction through the hybrid store abstraction.
/// - Return a fresh operator-facing status snapshot.
pub fn cmd_db_compact(args: &[String]) -> Result<(), AppError> {
    let backend = parse_backend(args)?;
    let (store, root) = open_store(backend)?;

    store.compact_index().map_err(|err| {
        AppError::with_source(
            ErrorCode::LedgerInvalid,
            format!(
                "Failed to compact metadata index at {} using {} backend",
                root.display(),
                backend_label(backend)
            ),
            err,
        )
    })?;

    let status = build_status(backend, &root)?;
    emit_serialized(&status, output_format(args))
}

