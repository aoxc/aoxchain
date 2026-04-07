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

        let expected = canonical_block_envelope_hash_hex(
            self.height,
            &self.parent_hash_hex,
            &self.payload,
        )?;
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
