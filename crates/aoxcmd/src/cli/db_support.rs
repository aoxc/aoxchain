const DEFAULT_INDEX_BACKEND: IndexBackend = IndexBackend::Redb;

/// Operator-facing database status payload.
///
/// This payload is intentionally compact, deterministic, and suitable for:
/// - shell automation,
/// - CI verification steps,
/// - operator diagnostics,
/// - post-action audit evidence collection.
///
/// The structure avoids exposing internal implementation details that may
/// change over time while preserving the minimum signals needed by operators.
#[derive(Debug, Serialize)]
struct DbStatus {
    backend: String,
    db_root: String,
    ipfs_object_count: usize,
    has_index_snapshot: bool,
    has_index_journal: bool,
}

/// Returns the stable operator-facing backend label.
fn backend_label(backend: IndexBackend) -> &'static str {
    match backend {
        IndexBackend::Sqlite => "sqlite",
        IndexBackend::Redb => "redb",
    }
}

/// Parses the requested metadata index backend from CLI arguments.
///
/// Accepted values:
/// - `sqlite`
/// - `redb`
///
/// Default behavior:
/// - Falls back to the AOXC canonical default backend when `--backend` is omitted.
///
/// Validation behavior:
/// - Rejects unknown values with a stable usage error.
fn parse_backend(args: &[String]) -> Result<IndexBackend, AppError> {
    let fallback = backend_label(DEFAULT_INDEX_BACKEND).to_string();

    match arg_value(args, "--backend")
        .unwrap_or(fallback)
        .to_ascii_lowercase()
        .as_str()
    {
        "sqlite" => Ok(IndexBackend::Sqlite),
        "redb" => Ok(IndexBackend::Redb),
        value => Err(AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Invalid --backend value '{value}'. Use sqlite|redb."),
        )),
    }
}

/// Resolves the effective AOXC database root beneath the effective AOXC home.
///
/// Path contract:
/// - The effective AOXC home is resolved by `resolve_home()`.
/// - The database surface is anchored under `runtime/db`.
///
/// Examples:
/// - `/home/<user>/.aoxc/runtime/db`
/// - `/home/<user>/.aoxc/.test/<label>/runtime/db`
///
/// Operational guarantee:
/// - The required AOXC layout is materialized before the root is returned so
///   downstream database commands operate on a prepared filesystem surface.
fn db_root() -> Result<PathBuf, AppError> {
    let home = resolve_home()?;
    ensure_layout(&home)?;
    Ok(home.join("runtime").join("db"))
}

/// Opens the hybrid data store for the selected backend under the effective root.
///
/// Error policy:
/// - Fails with a filesystem-oriented application error when the backend cannot
///   be opened for the resolved AOXC root.
fn open_store(backend: IndexBackend) -> Result<(HybridDataStore, PathBuf), AppError> {
    let root = db_root()?;
    let store = HybridDataStore::new(&root, backend).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!(
                "Failed to open {} data store at {}",
                backend_label(backend),
                root.display()
            ),
            err,
        )
    })?;

    Ok((store, root))
}

/// Parses a required unsigned integer CLI argument.
///
/// Validation behavior:
/// - Rejects missing values.
/// - Rejects non-numeric values.
/// - Preserves the CLI flag name in operator-visible error text.
fn parse_u64_arg(args: &[String], flag: &str) -> Result<u64, AppError> {
    arg_value(args, flag)
        .ok_or_else(|| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Missing required {flag} value"),
            )
        })?
        .parse::<u64>()
        .map_err(|_| {
            AppError::new(
                ErrorCode::UsageInvalidArguments,
                format!("Invalid numeric value for {flag}"),
            )
        })
}

/// Parses a required string CLI argument.
///
/// Validation behavior:
/// - Rejects a missing flag value.
/// - Preserves the original flag name in the emitted error.
fn parse_required_arg(args: &[String], flag: &str) -> Result<String, AppError> {
    arg_value(args, flag).ok_or_else(|| {
        AppError::new(
            ErrorCode::UsageInvalidArguments,
            format!("Missing required {flag} value"),
        )
    })
}

/// Counts persisted IPFS objects beneath the effective database root.
///
/// Design notes:
/// - The absence of the `ipfs/` directory is treated as a valid empty state.
/// - The count reflects materialized directory entries only.
/// - The function intentionally avoids recursive traversal to preserve
///   predictability and operator-facing performance.
fn count_ipfs_objects(root: &Path) -> Result<usize, AppError> {
    let ipfs_dir = root.join("ipfs");
    if !ipfs_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(&ipfs_dir).map_err(|err| {
        AppError::with_source(
            ErrorCode::FilesystemIoFailed,
            format!("Failed to scan {}", ipfs_dir.display()),
            err,
        )
    })?;

    Ok(entries.filter_map(Result::ok).count())
}

/// Builds the operator-facing database status payload for the effective root.
///
/// Compatibility note:
/// - `has_index_snapshot` and `has_index_journal` remain exposed as stable
///   diagnostics fields because operator tooling may already consume them,
///   regardless of the selected embedded metadata backend.
fn build_status(backend: IndexBackend, root: &Path) -> Result<DbStatus, AppError> {
    Ok(DbStatus {
        backend: backend_label(backend).to_string(),
        db_root: root.display().to_string(),
        ipfs_object_count: count_ipfs_objects(root)?,
        has_index_snapshot: root.join("index").join("snapshot.json").exists(),
        has_index_journal: root.join("index").join("journal.log").exists(),
    })
}
