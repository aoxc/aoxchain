pub mod contracts;
pub mod store;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions, create_dir_all};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Canonical content domain separator for AOXC block hashing.
const BLOCK_HASH_DOMAIN: &[u8] = b"AOXC_BLOCK_V1";

/// Canonical content domain separator for AOXC content-addressed blob storage.
const BLOB_CID_DOMAIN: &[u8] = b"AOXC_IPLD_BLOCK_V1";

/// Represents a normalized block payload envelope stored by the data layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockEnvelope {
    pub height: u64,
    pub block_hash_hex: String,
    pub parent_hash_hex: String,
    pub payload: Vec<u8>,
}

impl BlockEnvelope {
    /// Validates the structural and integrity properties of the block envelope.
    pub fn validate(&self) -> Result<(), DataError> {
        if self.height == 0 {
            return Err(DataError::InvalidInput(
                "block height must be greater than zero".to_owned(),
            ));
        }

        validate_hash_hex("block_hash_hex", &self.block_hash_hex)?;
        validate_hash_hex("parent_hash_hex", &self.parent_hash_hex)?;

        let expected = canonical_block_hash_hex(self.height, &self.parent_hash_hex, &self.payload)?;
        if self.block_hash_hex != expected {
            return Err(DataError::Integrity(format!(
                "block hash mismatch: expected '{}', got '{}'",
                expected, self.block_hash_hex
            )));
        }

        Ok(())
    }
}

/// Represents the storage receipt of a content-addressed block object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpfsRecord {
    pub cid: String,
    pub size: usize,
}

/// Persistent metadata used by the height and hash index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockMeta {
    pub height: u64,
    pub cid: String,
    pub block_hash_hex: String,
    pub parent_hash_hex: String,
    pub created_at_unix: u64,
}

impl BlockMeta {
    pub fn validate(&self) -> Result<(), DataError> {
        if self.height == 0 {
            return Err(DataError::InvalidInput(
                "metadata height must be greater than zero".to_owned(),
            ));
        }

        if self.cid.trim().is_empty() {
            return Err(DataError::InvalidInput(
                "metadata cid must not be empty".to_owned(),
            ));
        }

        validate_hash_hex("block_hash_hex", &self.block_hash_hex)?;
        validate_hash_hex("parent_hash_hex", &self.parent_hash_hex)?;
        Ok(())
    }
}

/// Declares the logical index implementation requested by the caller.
///
/// The current implementation uses the same hardened journaled index engine for
/// both variants. The enum is retained to preserve caller-facing flexibility and
/// future backend specialization without forcing API churn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexBackend {
    Sqlite,
    Redb,
}

/// Errors emitted by the AOXC data layer.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DataError {
    #[error("io error at '{path}': {reason}")]
    Io { path: String, reason: String },

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("not found")]
    NotFound,

    #[error("corrupted data: {0}")]
    Corrupted(String),

    #[error("integrity violation: {0}")]
    Integrity(String),

    #[error("concurrency failure: {0}")]
    Concurrency(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

/// Content-addressed blob storage boundary.
pub trait BlobStore {
    fn put_block(&self, block: &BlockEnvelope) -> Result<IpfsRecord, DataError>;
    fn get_block(&self, cid: &str) -> Result<BlockEnvelope, DataError>;
}

/// Metadata index boundary.
pub trait IndexStore {
    fn put_meta(&self, meta: &BlockMeta) -> Result<(), DataError>;
    fn get_by_height(&self, height: u64) -> Result<BlockMeta, DataError>;
    fn get_by_hash(&self, block_hash_hex: &str) -> Result<BlockMeta, DataError>;
    fn compact(&self) -> Result<(), DataError>;
}

/// Filesystem-backed content-addressed blob store.
///
/// Objects are persisted as JSON documents under a deterministic CID-derived
/// file name. Read operations verify both CID integrity and block integrity.
#[derive(Debug, Clone)]
pub struct FsCasBlobStore {
    root: PathBuf,
}

impl FsCasBlobStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, DataError> {
        let root = root.as_ref().to_path_buf();
        create_dir_all_strict(&root)?;
        Ok(Self { root })
    }

    fn block_path(&self, cid: &str) -> PathBuf {
        self.root.join(format!("{cid}.json"))
    }

    fn synthetic_cid(payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(BLOB_CID_DOMAIN);
        hasher.update(payload);
        let digest = hasher.finalize();
        format!("bafy{}", hex::encode(&digest[..16]))
    }
}

impl BlobStore for FsCasBlobStore {
    fn put_block(&self, block: &BlockEnvelope) -> Result<IpfsRecord, DataError> {
        block.validate()?;

        let encoded =
            serde_json::to_vec(block).map_err(|err| DataError::Serialization(err.to_string()))?;
        let cid = Self::synthetic_cid(&encoded);
        let path = self.block_path(&cid);

        if path.exists() {
            let existing = self.get_block(&cid)?;
            if existing != *block {
                return Err(DataError::Integrity(format!(
                    "content collision detected for cid '{}'",
                    cid
                )));
            }

            return Ok(IpfsRecord {
                cid,
                size: block.payload.len(),
            });
        }

        write_atomic(&path, &encoded)?;
        Ok(IpfsRecord {
            cid,
            size: block.payload.len(),
        })
    }

    fn get_block(&self, cid: &str) -> Result<BlockEnvelope, DataError> {
        let path = self.block_path(cid);
        if !path.exists() {
            return Err(DataError::NotFound);
        }

        let data = read_file_strict(&path)?;
        let expected_cid = Self::synthetic_cid(&data);
        if expected_cid != cid {
            return Err(DataError::Integrity(format!(
                "blob cid mismatch: expected '{}', got '{}'",
                expected_cid, cid
            )));
        }

        let block: BlockEnvelope = serde_json::from_slice(&data)
            .map_err(|err| DataError::Corrupted(format!("invalid block blob: {err}")))?;
        block.validate()?;
        Ok(block)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after unix epoch")
            .as_nanos();

        let dir = std::env::temp_dir().join(format!("aoxcdata_{label}_{nanos}"));
        fs::create_dir_all(&dir).expect("temporary directory must be created");
        dir
    }

    fn make_block(height: u64, parent_hash_hex: &str, payload: &[u8]) -> BlockEnvelope {
        let block_hash_hex = canonical_block_hash_hex(height, parent_hash_hex, payload)
            .expect("block hash computation must succeed");

        BlockEnvelope {
            height,
            block_hash_hex,
            parent_hash_hex: parent_hash_hex.to_owned(),
            payload: payload.to_vec(),
        }
    }

    #[test]
    fn block_roundtrip_by_height_succeeds() {
        let dir = unique_temp_dir("roundtrip");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(1, &"00".repeat(32), b"hello");
        let meta = store.put_block(&block).expect("put block must succeed");
        assert!(meta.cid.starts_with("bafy"));

        let loaded = store
            .get_block_by_height(1)
            .expect("block retrieval by height must succeed");
        assert_eq!(loaded, block);

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn duplicate_height_is_rejected() {
        let dir = unique_temp_dir("dup_height");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block_a = make_block(1, &"00".repeat(32), b"alpha");
        let block_b = make_block(1, &"00".repeat(32), b"beta");

        store.put_block(&block_a).expect("first block must succeed");

        let err = store
            .put_block(&block_b)
            .expect_err("duplicate height must fail");

        match err {
            DataError::AlreadyExists(message) => assert!(message.contains("height")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn duplicate_hash_is_rejected() {
        let dir = unique_temp_dir("dup_hash");
        let index = FileMetaIndexStore::new(dir.join("index"), IndexBackend::Redb)
            .expect("index init must succeed");

        let meta_a = BlockMeta {
            height: 1,
            cid: "bafyalpha".to_owned(),
            block_hash_hex: "aa".repeat(32),
            parent_hash_hex: "00".repeat(32),
            created_at_unix: 1,
        };

        let meta_b = BlockMeta {
            height: 2,
            cid: "bafybeta".to_owned(),
            block_hash_hex: "aa".repeat(32),
            parent_hash_hex: "11".repeat(32),
            created_at_unix: 2,
        };

        index.put_meta(&meta_a).expect("first meta must succeed");

        let err = index
            .put_meta(&meta_b)
            .expect_err("duplicate hash must fail");

        match err {
            DataError::AlreadyExists(message) => assert!(message.contains("hash")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn invalid_block_hash_is_rejected() {
        let block = BlockEnvelope {
            height: 1,
            block_hash_hex: "ff".repeat(32),
            parent_hash_hex: "00".repeat(32),
            payload: b"payload".to_vec(),
        };

        let err = block.validate().expect_err("invalid block hash must fail");
        match err {
            DataError::Integrity(message) => assert!(message.contains("block hash mismatch")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn tampered_blob_is_detected_by_integrity_verification() {
        let dir = unique_temp_dir("tampered_blob");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(1, &"00".repeat(32), b"hello");
        let meta = store.put_block(&block).expect("put block must succeed");

        let path = store.ipfs.block_path(&meta.cid);
        write_atomic(&path, b"{\"tampered\":true}").expect("tampered write must succeed");

        let err = store
            .ipfs
            .get_block(&meta.cid)
            .expect_err("tampered blob must fail");

        match err {
            DataError::Integrity(_) | DataError::Corrupted(_) => {}
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn journal_replay_restores_index_state() {
        let dir = unique_temp_dir("journal_replay");
        let index_dir = dir.join("index");

        {
            let index = FileMetaIndexStore::new(&index_dir, IndexBackend::Sqlite)
                .expect("index init must succeed");

            let meta = BlockMeta {
                height: 7,
                cid: "bafyrestore".to_owned(),
                block_hash_hex: "ab".repeat(32),
                parent_hash_hex: "cd".repeat(32),
                created_at_unix: 42,
            };

            index.put_meta(&meta).expect("meta insert must succeed");
        }

        let reopened = FileMetaIndexStore::new(&index_dir, IndexBackend::Sqlite)
            .expect("index reopen must succeed");

        let loaded = reopened
            .get_by_height(7)
            .expect("replayed entry must exist");

        assert_eq!(loaded.cid, "bafyrestore");
        assert_eq!(loaded.block_hash_hex, "ab".repeat(32));

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn corrupted_journal_is_rejected_on_open() {
        let dir = unique_temp_dir("corrupted_journal");
        let index_dir = dir.join("index");
        let index = FileMetaIndexStore::new(&index_dir, IndexBackend::Redb)
            .expect("index init must succeed");

        index.compact().expect("compact must succeed");

        let journal_path = index_dir.join("meta_redb_journal.log");
        write_atomic(&journal_path, b"{invalid json\n")
            .expect("corrupt journal write must succeed");

        let err = FileMetaIndexStore::new(&index_dir, IndexBackend::Redb)
            .expect_err("corrupted journal must fail");

        match err {
            DataError::Corrupted(message) => assert!(message.contains("invalid journal record")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn non_genesis_block_requires_parent_height_to_exist() {
        let dir = unique_temp_dir("missing_parent");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let block = make_block(2, &"ab".repeat(32), b"orphan");
        let err = store
            .put_block(&block)
            .expect_err("orphan block must be rejected");

        match err {
            DataError::InvalidInput(message) => assert!(message.contains("missing parent block")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn non_genesis_block_requires_parent_hash_match() {
        let dir = unique_temp_dir("parent_mismatch");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let genesis = make_block(1, &"00".repeat(32), b"genesis");
        store.put_block(&genesis).expect("genesis put must succeed");

        let bad_child = make_block(2, &"ff".repeat(32), b"child");
        let err = store
            .put_block(&bad_child)
            .expect_err("child with mismatched parent hash must fail");

        match err {
            DataError::Integrity(message) => assert!(message.contains("parent hash mismatch")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn genesis_block_requires_zero_parent_hash() {
        let dir = unique_temp_dir("genesis_parent");
        let store = HybridDataStore::new(&dir, IndexBackend::Sqlite).expect("store init");

        let bad_genesis = make_block(1, &"11".repeat(32), b"genesis");
        let err = store
            .put_block(&bad_genesis)
            .expect_err("genesis with non-zero parent hash must fail");

        match err {
            DataError::InvalidInput(message) => {
                assert!(message.contains("genesis block parent hash"))
            }
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }
}
