use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockEnvelope {
    pub height: u64,
    pub block_hash_hex: String,
    pub parent_hash_hex: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IpfsRecord {
    pub cid: String,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockMeta {
    pub height: u64,
    pub cid: String,
    pub block_hash_hex: String,
    pub parent_hash_hex: String,
    pub created_at_unix: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexBackend {
    Sqlite,
    Redb,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("not found")]
    NotFound,
}

pub trait BlobStore {
    fn put_block(&self, block: &BlockEnvelope) -> Result<IpfsRecord, DataError>;
    fn get_block(&self, cid: &str) -> Result<BlockEnvelope, DataError>;
}

pub trait IndexStore {
    fn put_meta(&self, meta: &BlockMeta) -> Result<(), DataError>;
    fn get_by_height(&self, height: u64) -> Result<BlockMeta, DataError>;
}

#[derive(Debug, Clone)]
pub struct IpfsBlobStore {
    root: PathBuf,
}

impl IpfsBlobStore {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, DataError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).map_err(|e| DataError::Io(e.to_string()))?;
        Ok(Self { root })
    }

    fn block_path(&self, cid: &str) -> PathBuf {
        self.root.join(format!("{cid}.json"))
    }

    fn synthetic_cid(payload: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_IPLD_BLOCK_V1");
        hasher.update(payload);
        let digest = hasher.finalize();
        format!("bafy{}", hex::encode(&digest[..16]))
    }
}

impl BlobStore for IpfsBlobStore {
    fn put_block(&self, block: &BlockEnvelope) -> Result<IpfsRecord, DataError> {
        let encoded =
            serde_json::to_vec(block).map_err(|e| DataError::Serialization(e.to_string()))?;
        let cid = Self::synthetic_cid(&encoded);
        let path = self.block_path(&cid);
        fs::write(path, encoded).map_err(|e| DataError::Io(e.to_string()))?;
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

        let data = fs::read(path).map_err(|e| DataError::Io(e.to_string()))?;
        serde_json::from_slice(&data).map_err(|e| DataError::Serialization(e.to_string()))
    }
}

#[derive(Debug, Clone)]
struct JsonIndexStore {
    path: PathBuf,
}

impl JsonIndexStore {
    fn new(path: impl AsRef<Path>) -> Result<Self, DataError> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| DataError::Io(e.to_string()))?;
        }

        if !path.exists() {
            fs::write(&path, "{}".as_bytes()).map_err(|e| DataError::Io(e.to_string()))?;
        }

        Ok(Self { path })
    }

    fn load_map(&self) -> Result<BTreeMap<u64, BlockMeta>, DataError> {
        let data = fs::read(&self.path).map_err(|e| DataError::Io(e.to_string()))?;
        serde_json::from_slice(&data).map_err(|e| DataError::Serialization(e.to_string()))
    }

    fn save_map(&self, map: &BTreeMap<u64, BlockMeta>) -> Result<(), DataError> {
        let data =
            serde_json::to_vec_pretty(map).map_err(|e| DataError::Serialization(e.to_string()))?;
        fs::write(&self.path, data).map_err(|e| DataError::Io(e.to_string()))
    }
}

impl IndexStore for JsonIndexStore {
    fn put_meta(&self, meta: &BlockMeta) -> Result<(), DataError> {
        let mut map = self.load_map()?;
        map.insert(meta.height, meta.clone());
        self.save_map(&map)
    }

    fn get_by_height(&self, height: u64) -> Result<BlockMeta, DataError> {
        let map = self.load_map()?;
        map.get(&height).cloned().ok_or(DataError::NotFound)
    }
}

enum IndexStoreImpl {
    Sqlite(JsonIndexStore),
    Redb(JsonIndexStore),
}

impl IndexStore for IndexStoreImpl {
    fn put_meta(&self, meta: &BlockMeta) -> Result<(), DataError> {
        match self {
            Self::Sqlite(store) => store.put_meta(meta),
            Self::Redb(store) => store.put_meta(meta),
        }
    }

    fn get_by_height(&self, height: u64) -> Result<BlockMeta, DataError> {
        match self {
            Self::Sqlite(store) => store.get_by_height(height),
            Self::Redb(store) => store.get_by_height(height),
        }
    }
}

pub struct HybridDataStore {
    pub ipfs: IpfsBlobStore,
    index: IndexStoreImpl,
}

impl HybridDataStore {
    pub fn new(base_dir: impl AsRef<Path>, backend: IndexBackend) -> Result<Self, DataError> {
        let base = base_dir.as_ref();
        fs::create_dir_all(base).map_err(|e| DataError::Io(e.to_string()))?;

        let ipfs = IpfsBlobStore::new(base.join("ipfs"))?;

        let index = match backend {
            IndexBackend::Sqlite => {
                IndexStoreImpl::Sqlite(JsonIndexStore::new(base.join("state_sqlite_index.json"))?)
            }
            IndexBackend::Redb => {
                IndexStoreImpl::Redb(JsonIndexStore::new(base.join("state_redb_index.json"))?)
            }
        };

        Ok(Self { ipfs, index })
    }

    pub fn put_block(&self, block: &BlockEnvelope) -> Result<BlockMeta, DataError> {
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
}

fn current_unix_ts() -> Result<u64, DataError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| DataError::Io(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_policy_roundtrip() {
        let temp = std::env::temp_dir().join("aoxcdata_sqlite_policy_test");
        let _ = fs::remove_dir_all(&temp);

        let store = HybridDataStore::new(&temp, IndexBackend::Sqlite).expect("store init");

        let block = BlockEnvelope {
            height: 1,
            block_hash_hex: "aa".repeat(32),
            parent_hash_hex: "00".repeat(32),
            payload: b"hello".to_vec(),
        };

        let meta = store.put_block(&block).expect("put block");
        assert!(meta.cid.starts_with("bafy"));

        let loaded = store.get_block_by_height(1).expect("load block");
        assert_eq!(loaded.height, 1);
        assert_eq!(loaded.payload, b"hello".to_vec());
    }
}
