#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SnapshotState {
    by_height: BTreeMap<u64, BlockMeta>,
    by_hash: BTreeMap<String, u64>,
}

impl From<&IndexState> for SnapshotState {
    fn from(value: &IndexState) -> Self {
        Self {
            by_height: value.by_height.clone(),
            by_hash: value.by_hash.clone(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct IndexState {
    by_height: BTreeMap<u64, BlockMeta>,
    by_hash: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum JournalRecord {
    PutMeta { meta: BlockMeta },
}

#[derive(Debug)]
struct IndexStoreInner {
    snapshot_path: PathBuf,
    journal_path: PathBuf,
    state: IndexState,
}

/// Hardened journaled index store.
///
/// Persistence model:
/// - durable append-only journal for mutation recording
/// - materialized snapshot for bounded replay time
/// - in-memory state guarded by a mutex for consistency
#[derive(Debug)]
pub struct FileMetaIndexStore {
    inner: Mutex<IndexStoreInner>,
}

impl FileMetaIndexStore {
    pub fn new(base_dir: impl AsRef<Path>, backend: IndexBackend) -> Result<Self, DataError> {
        let base_dir = base_dir.as_ref();
        create_dir_all_strict(base_dir)?;

        let (snapshot_name, journal_name) = match backend {
            IndexBackend::Sqlite => ("meta_sqlite_snapshot.json", "meta_sqlite_journal.log"),
            IndexBackend::Redb => ("meta_redb_snapshot.json", "meta_redb_journal.log"),
        };

        let snapshot_path = base_dir.join(snapshot_name);
        let journal_path = base_dir.join(journal_name);

        if !snapshot_path.exists() {
            let initial_snapshot = serde_json::to_vec_pretty(&SnapshotState::default())
                .map_err(|err| DataError::Serialization(err.to_string()))?;
            write_atomic(&snapshot_path, &initial_snapshot)?;
        }

        if !journal_path.exists() {
            write_atomic(&journal_path, b"")?;
        }

        let state = load_index_state(&snapshot_path, &journal_path)?;
        Ok(Self {
            inner: Mutex::new(IndexStoreInner {
                snapshot_path,
                journal_path,
                state,
            }),
        })
    }
}

impl IndexStore for FileMetaIndexStore {
    fn put_meta(&self, meta: &BlockMeta) -> Result<(), DataError> {
        meta.validate()?;

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| DataError::Concurrency("index store mutex poisoned".to_owned()))?;

        if guard.state.by_height.contains_key(&meta.height) {
            return Err(DataError::AlreadyExists(format!(
                "block height '{}' already exists",
                meta.height
            )));
        }

        if guard.state.by_hash.contains_key(&meta.block_hash_hex) {
            return Err(DataError::AlreadyExists(format!(
                "block hash '{}' already exists",
                meta.block_hash_hex
            )));
        }

        let record = JournalRecord::PutMeta { meta: meta.clone() };
        append_journal_record(&guard.journal_path, &record)?;

        guard
            .state
            .by_hash
            .insert(meta.block_hash_hex.clone(), meta.height);
        guard.state.by_height.insert(meta.height, meta.clone());

        Ok(())
    }

    fn get_by_height(&self, height: u64) -> Result<BlockMeta, DataError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| DataError::Concurrency("index store mutex poisoned".to_owned()))?;

        guard
            .state
            .by_height
            .get(&height)
            .cloned()
            .ok_or(DataError::NotFound)
    }

    fn get_by_hash(&self, block_hash_hex: &str) -> Result<BlockMeta, DataError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| DataError::Concurrency("index store mutex poisoned".to_owned()))?;

        let height = guard
            .state
            .by_hash
            .get(block_hash_hex)
            .copied()
            .ok_or(DataError::NotFound)?;

        guard
            .state
            .by_height
            .get(&height)
            .cloned()
            .ok_or(DataError::NotFound)
    }

    fn compact(&self) -> Result<(), DataError> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| DataError::Concurrency("index store mutex poisoned".to_owned()))?;

        let snapshot = SnapshotState::from(&guard.state);
        let encoded = serde_json::to_vec_pretty(&snapshot)
            .map_err(|err| DataError::Serialization(err.to_string()))?;
        write_atomic(&guard.snapshot_path, &encoded)?;
        write_atomic(&guard.journal_path, b"")?;
        Ok(())
    }
}

/// Hybrid data store combining content-addressed blob persistence with a durable
/// metadata index.
#[derive(Debug)]
pub struct HybridDataStore {
    pub ipfs: FsCasBlobStore,
    index: FileMetaIndexStore,
}

impl HybridDataStore {
    pub fn new(base_dir: impl AsRef<Path>, backend: IndexBackend) -> Result<Self, DataError> {
        let base_dir = base_dir.as_ref();
        create_dir_all_strict(base_dir)?;

        let ipfs = FsCasBlobStore::new(base_dir.join("ipfs"))?;
        let index = FileMetaIndexStore::new(base_dir.join("index"), backend)?;
        Ok(Self { ipfs, index })
    }

    pub fn put_block(&self, block: &BlockEnvelope) -> Result<BlockMeta, DataError> {
        block.validate()?;
        self.validate_chain_link(block)?;

        let ipfs_record = self.ipfs.put_block(block)?;
        let meta = BlockMeta {
            height: block.height,
            cid: ipfs_record.cid,
            block_hash_hex: block.block_hash_hex.clone(),
            parent_hash_hex: block.parent_hash_hex.clone(),
            created_at_unix: current_unix_ts()?,
        };

        self.index.put_meta(&meta)?;
        Ok(meta)
    }

    pub fn get_block_by_height(&self, height: u64) -> Result<BlockEnvelope, DataError> {
        let meta = self.index.get_by_height(height)?;
        self.ipfs.get_block(&meta.cid)
    }

    pub fn get_block_by_hash(&self, block_hash_hex: &str) -> Result<BlockEnvelope, DataError> {
        let meta = self.index.get_by_hash(block_hash_hex)?;
        self.ipfs.get_block(&meta.cid)
    }

    pub fn compact_index(&self) -> Result<(), DataError> {
        self.index.compact()
    }

    fn validate_chain_link(&self, block: &BlockEnvelope) -> Result<(), DataError> {
        if block.height == 1 {
            let expected_genesis_parent = "00".repeat(32);
            if block.parent_hash_hex != expected_genesis_parent {
                return Err(DataError::InvalidInput(format!(
                    "genesis block parent hash must be '{}'",
                    expected_genesis_parent
                )));
            }
            return Ok(());
        }

        let previous_height = block.height.checked_sub(1).ok_or_else(|| {
            DataError::InvalidInput("block height underflow while validating chain link".to_owned())
        })?;

        let previous = self
            .index
            .get_by_height(previous_height)
            .map_err(|err| match err {
                DataError::NotFound => DataError::InvalidInput(format!(
                    "missing parent block at height '{}' for block '{}'",
                    previous_height, block.height
                )),
                other => other,
            })?;

        if block.parent_hash_hex != previous.block_hash_hex {
            return Err(DataError::Integrity(format!(
                "parent hash mismatch at height '{}': expected '{}', got '{}'",
                block.height, previous.block_hash_hex, block.parent_hash_hex
            )));
        }

        Ok(())
    }
}

fn current_unix_ts() -> Result<u64, DataError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| DataError::Io {
            path: "system_clock".to_owned(),
            reason: err.to_string(),
        })
}

fn validate_hash_hex(field: &str, value: &str) -> Result<(), DataError> {
    if value.len() != 64 {
        return Err(DataError::InvalidInput(format!(
            "{field} must be 64 hexadecimal characters"
        )));
    }

    if !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(DataError::InvalidInput(format!(
            "{field} must contain only hexadecimal characters"
        )));
    }

    Ok(())
}

fn canonical_block_hash_hex(
    height: u64,
    parent_hash_hex: &str,
    payload: &[u8],
) -> Result<String, DataError> {
    validate_hash_hex("parent_hash_hex", parent_hash_hex)?;

    let parent_hash = hex::decode(parent_hash_hex).map_err(|err| {
        DataError::InvalidInput(format!("parent_hash_hex could not be decoded: {err}"))
    })?;

    let mut hasher = Sha256::new();
    hasher.update(BLOCK_HASH_DOMAIN);
    hasher.update(height.to_le_bytes());
    hasher.update(parent_hash);
    hasher.update((payload.len() as u64).to_le_bytes());
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

fn create_dir_all_strict(path: &Path) -> Result<(), DataError> {
    create_dir_all(path).map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })
}

fn read_file_strict(path: &Path) -> Result<Vec<u8>, DataError> {
    let mut file = File::open(path).map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;
    Ok(data)
}

fn write_atomic(path: &Path, data: &[u8]) -> Result<(), DataError> {
    let parent = path.parent().ok_or_else(|| DataError::Io {
        path: path.display().to_string(),
        reason: "target path has no parent directory".to_owned(),
    })?;
    create_dir_all_strict(parent)?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    let tmp_path = parent.join(format!("{file_name}.tmp"));

    if tmp_path.exists() {
        fs::remove_file(&tmp_path).map_err(|err| DataError::Io {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    {
        let mut file = File::create(&tmp_path).map_err(|err| DataError::Io {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        file.write_all(data).map_err(|err| DataError::Io {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
        file.sync_all().map_err(|err| DataError::Io {
            path: tmp_path.display().to_string(),
            reason: err.to_string(),
        })?;
    }

    fs::rename(&tmp_path, path).map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;

    Ok(())
}

fn append_journal_record(path: &Path, record: &JournalRecord) -> Result<(), DataError> {
    let encoded =
        serde_json::to_vec(record).map_err(|err| DataError::Serialization(err.to_string()))?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| DataError::Io {
            path: path.display().to_string(),
            reason: err.to_string(),
        })?;

    file.write_all(&encoded).map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;
    file.write_all(b"\n").map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;
    file.sync_all().map_err(|err| DataError::Io {
        path: path.display().to_string(),
        reason: err.to_string(),
    })?;

    Ok(())
}

fn load_index_state(snapshot_path: &Path, journal_path: &Path) -> Result<IndexState, DataError> {
    let snapshot_data = read_file_strict(snapshot_path)?;
    let snapshot: SnapshotState = serde_json::from_slice(&snapshot_data)
        .map_err(|err| DataError::Corrupted(format!("invalid snapshot: {err}")))?;

    let mut state = IndexState {
        by_height: snapshot.by_height,
        by_hash: snapshot.by_hash,
    };

    let file = File::open(journal_path).map_err(|err| DataError::Io {
        path: journal_path.display().to_string(),
        reason: err.to_string(),
    })?;
    let reader = BufReader::new(file);

    for (line_no, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|err| DataError::Io {
            path: journal_path.display().to_string(),
            reason: err.to_string(),
        })?;

        if line.trim().is_empty() {
            continue;
        }

        let record: JournalRecord = serde_json::from_str(&line).map_err(|err| {
            DataError::Corrupted(format!(
                "invalid journal record at line {}: {}",
                line_no + 1,
                err
            ))
        })?;

        match record {
            JournalRecord::PutMeta { meta } => {
                meta.validate()?;

                if state.by_height.contains_key(&meta.height) {
                    return Err(DataError::Corrupted(format!(
                        "duplicate block height '{}' discovered during journal replay at line {}",
                        meta.height,
                        line_no + 1
                    )));
                }

                if state.by_hash.contains_key(&meta.block_hash_hex) {
                    return Err(DataError::Corrupted(format!(
                        "duplicate block hash '{}' discovered during journal replay at line {}",
                        meta.block_hash_hex,
                        line_no + 1
                    )));
                }

                state
                    .by_hash
                    .insert(meta.block_hash_hex.clone(), meta.height);
                state.by_height.insert(meta.height, meta);
            }
        }
    }

    Ok(state)
}

