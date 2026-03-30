// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::DataError;
use crate::store::column_families::{ColumnFamily, all_column_families};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

type KvKey = (ColumnFamily, Vec<u8>);
type KvValue = Vec<u8>;
type KvMap = BTreeMap<KvKey, KvValue>;
type SharedKvMap = Arc<RwLock<KvMap>>;

/// Atomic KV write operation.
///
/// The operation model is intentionally explicit and minimal. Deletion remains a
/// dedicated method because its operational semantics differ materially from a
/// write/update action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvWriteOp {
    pub column_family: ColumnFamily,
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl KvWriteOp {
    #[must_use]
    pub fn put(column_family: ColumnFamily, key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            column_family,
            key,
            value,
        }
    }
}

/// Deterministic KV storage boundary.
///
/// The trait intentionally exposes explicit failure semantics. Storage layers
/// must not silently ignore write failures or poisoned synchronization state.
pub trait KvDb: Send + Sync {
    fn put(&self, cf: ColumnFamily, key: Vec<u8>, value: Vec<u8>) -> Result<(), DataError>;
    fn get(&self, cf: ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, DataError>;
    fn delete(&self, cf: ColumnFamily, key: &[u8]) -> Result<(), DataError>;
    fn apply_batch(&self, ops: &[KvWriteOp]) -> Result<(), DataError>;
}

/// In-memory deterministic KV implementation.
///
/// Suitable for tests and ephemeral execution paths. This implementation is
/// intentionally strict about lock poisoning and does not degrade into silent
/// failure behavior.
#[derive(Debug, Default, Clone)]
pub struct InMemoryKvDb {
    inner: SharedKvMap,
}

impl InMemoryKvDb {
    fn write_guard(&self) -> Result<std::sync::RwLockWriteGuard<'_, KvMap>, DataError> {
        self.inner
            .write()
            .map_err(|_| DataError::Concurrency("in-memory kv write lock poisoned".to_owned()))
    }

    fn read_guard(&self) -> Result<std::sync::RwLockReadGuard<'_, KvMap>, DataError> {
        self.inner
            .read()
            .map_err(|_| DataError::Concurrency("in-memory kv read lock poisoned".to_owned()))
    }
}

impl KvDb for InMemoryKvDb {
    fn put(&self, cf: ColumnFamily, key: Vec<u8>, value: Vec<u8>) -> Result<(), DataError> {
        let mut guard = self.write_guard()?;
        guard.insert((cf, key), value);
        Ok(())
    }

    fn get(&self, cf: ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, DataError> {
        let guard = self.read_guard()?;
        Ok(guard.get(&(cf, key.to_vec())).cloned())
    }

    fn delete(&self, cf: ColumnFamily, key: &[u8]) -> Result<(), DataError> {
        let mut guard = self.write_guard()?;
        guard.remove(&(cf, key.to_vec()));
        Ok(())
    }

    fn apply_batch(&self, ops: &[KvWriteOp]) -> Result<(), DataError> {
        let mut guard = self.write_guard()?;
        for op in ops {
            guard.insert((op.column_family, op.key.clone()), op.value.clone());
        }
        Ok(())
    }
}

/// Filesystem-backed KV database.
///
/// Each entry is stored as:
/// `root/<column_family>/<sha256(key)>.json`
///
/// The on-disk format embeds the original key and value in hex encoding, which
/// enables read-time integrity checks and protects against accidental path/key
/// mismatch.
#[derive(Debug, Clone)]
pub struct FileKvDb {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredKvEntry {
    key_hex: String,
    value_hex: String,
}

impl FileKvDb {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, DataError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).map_err(|err| DataError::Io {
            path: root.display().to_string(),
            reason: err.to_string(),
        })?;

        for family in all_column_families() {
            let dir = root.join(family.as_str());
            fs::create_dir_all(&dir).map_err(|err| DataError::Io {
                path: dir.display().to_string(),
                reason: err.to_string(),
            })?;
        }

        Ok(Self { root })
    }

    fn entry_path(&self, cf: ColumnFamily, key: &[u8]) -> PathBuf {
        let digest = hex::encode(Sha256::digest(key));
        self.root.join(cf.as_str()).join(format!("{digest}.json"))
    }

    fn temp_entry_path(path: &Path) -> PathBuf {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("entry.json");
        path.with_file_name(format!("{file_name}.tmp"))
    }

    fn encode_entry(key: &[u8], value: &[u8]) -> Result<Vec<u8>, DataError> {
        let entry = StoredKvEntry {
            key_hex: hex::encode(key),
            value_hex: hex::encode(value),
        };

        serde_json::to_vec(&entry).map_err(|err| DataError::Serialization(err.to_string()))
    }

    fn write_atomic(path: &Path, data: &[u8]) -> Result<(), DataError> {
        let tmp = Self::temp_entry_path(path);

        {
            let mut file = File::create(&tmp).map_err(|err| DataError::Io {
                path: tmp.display().to_string(),
                reason: err.to_string(),
            })?;
            file.write_all(data).map_err(|err| DataError::Io {
                path: tmp.display().to_string(),
                reason: err.to_string(),
            })?;
            file.sync_all().map_err(|err| DataError::Io {
                path: tmp.display().to_string(),
                reason: err.to_string(),
            })?;
        }

        fs::rename(&tmp, path).map_err(|err| DataError::Io {
            path: path.display().to_string(),
            reason: err.to_string(),
        })
    }

    fn read_entry(&self, path: &Path, expected_key: &[u8]) -> Result<Option<Vec<u8>>, DataError> {
        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(path).map_err(|err| DataError::Io {
            path: path.display().to_string(),
            reason: err.to_string(),
        })?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|err| DataError::Io {
            path: path.display().to_string(),
            reason: err.to_string(),
        })?;

        let entry: StoredKvEntry =
            serde_json::from_slice(&data).map_err(|err| DataError::Corrupted(err.to_string()))?;

        let decoded_key = hex::decode(&entry.key_hex)
            .map_err(|err| DataError::Corrupted(format!("invalid key hex: {err}")))?;

        if decoded_key != expected_key {
            return Err(DataError::Integrity(
                "stored key does not match requested key".to_owned(),
            ));
        }

        let value = hex::decode(&entry.value_hex)
            .map_err(|err| DataError::Corrupted(format!("invalid value hex: {err}")))?;

        Ok(Some(value))
    }

    fn stage_batch(&self, ops: &[KvWriteOp]) -> Result<Vec<(PathBuf, PathBuf)>, DataError> {
        let mut staged = Vec::with_capacity(ops.len());

        for op in ops {
            let final_path = self.entry_path(op.column_family, &op.key);
            let tmp_path = Self::temp_entry_path(&final_path);
            let encoded = Self::encode_entry(&op.key, &op.value)?;

            {
                let mut file = File::create(&tmp_path).map_err(|err| DataError::Io {
                    path: tmp_path.display().to_string(),
                    reason: err.to_string(),
                })?;
                file.write_all(&encoded).map_err(|err| DataError::Io {
                    path: tmp_path.display().to_string(),
                    reason: err.to_string(),
                })?;
                file.sync_all().map_err(|err| DataError::Io {
                    path: tmp_path.display().to_string(),
                    reason: err.to_string(),
                })?;
            }

            staged.push((tmp_path, final_path));
        }

        Ok(staged)
    }

    fn commit_staged_batch(staged: &[(PathBuf, PathBuf)]) -> Result<(), DataError> {
        for (tmp_path, final_path) in staged {
            fs::rename(tmp_path, final_path).map_err(|err| DataError::Io {
                path: final_path.display().to_string(),
                reason: err.to_string(),
            })?;
        }

        Ok(())
    }

    fn cleanup_staged_batch(staged: &[(PathBuf, PathBuf)]) {
        for (tmp_path, _) in staged {
            let _ = fs::remove_file(tmp_path);
        }
    }
}

impl KvDb for FileKvDb {
    fn put(&self, cf: ColumnFamily, key: Vec<u8>, value: Vec<u8>) -> Result<(), DataError> {
        let path = self.entry_path(cf, &key);
        let encoded = Self::encode_entry(&key, &value)?;
        Self::write_atomic(&path, &encoded)
    }

    fn get(&self, cf: ColumnFamily, key: &[u8]) -> Result<Option<Vec<u8>>, DataError> {
        let path = self.entry_path(cf, key);
        self.read_entry(&path, key)
    }

    fn delete(&self, cf: ColumnFamily, key: &[u8]) -> Result<(), DataError> {
        let path = self.entry_path(cf, key);
        if !path.exists() {
            return Ok(());
        }

        fs::remove_file(&path).map_err(|err| DataError::Io {
            path: path.display().to_string(),
            reason: err.to_string(),
        })
    }

    fn apply_batch(&self, ops: &[KvWriteOp]) -> Result<(), DataError> {
        if ops.is_empty() {
            return Ok(());
        }

        let staged = self.stage_batch(ops)?;
        match Self::commit_staged_batch(&staged) {
            Ok(()) => Ok(()),
            Err(err) => {
                Self::cleanup_staged_batch(&staged);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aoxcdata_kv_{label}_{nanos}"));
        fs::create_dir_all(&dir).expect("temporary directory must be created");
        dir
    }

    #[test]
    fn in_memory_kv_roundtrip_succeeds() {
        let db = InMemoryKvDb::default();

        db.put(ColumnFamily::State, b"alpha".to_vec(), b"one".to_vec())
            .expect("put must succeed");

        let loaded = db
            .get(ColumnFamily::State, b"alpha")
            .expect("get must succeed");

        assert_eq!(loaded, Some(b"one".to_vec()));
    }

    #[test]
    fn file_kv_roundtrip_and_delete_succeed() {
        let dir = unique_temp_dir("roundtrip");
        let db = FileKvDb::new(&dir).expect("db init must succeed");

        db.put(
            ColumnFamily::Transactions,
            b"tx-1".to_vec(),
            b"payload".to_vec(),
        )
        .expect("put must succeed");

        let loaded = db
            .get(ColumnFamily::Transactions, b"tx-1")
            .expect("get must succeed");
        assert_eq!(loaded, Some(b"payload".to_vec()));

        db.delete(ColumnFamily::Transactions, b"tx-1")
            .expect("delete must succeed");

        let loaded = db
            .get(ColumnFamily::Transactions, b"tx-1")
            .expect("get must succeed");
        assert_eq!(loaded, None);

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn file_kv_batch_write_succeeds() {
        let dir = unique_temp_dir("batch");
        let db = FileKvDb::new(&dir).expect("db init must succeed");

        db.apply_batch(&[
            KvWriteOp::put(ColumnFamily::Blocks, b"k1".to_vec(), b"v1".to_vec()),
            KvWriteOp::put(ColumnFamily::Blocks, b"k2".to_vec(), b"v2".to_vec()),
        ])
        .expect("batch must succeed");

        assert_eq!(
            db.get(ColumnFamily::Blocks, b"k1")
                .expect("get must succeed"),
            Some(b"v1".to_vec())
        );
        assert_eq!(
            db.get(ColumnFamily::Blocks, b"k2")
                .expect("get must succeed"),
            Some(b"v2".to_vec())
        );

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn file_kv_detects_tampered_key() {
        let dir = unique_temp_dir("tamper");
        let db = FileKvDb::new(&dir).expect("db init must succeed");

        db.put(
            ColumnFamily::Receipts,
            b"expected".to_vec(),
            b"value".to_vec(),
        )
        .expect("put must succeed");

        let path = db.entry_path(ColumnFamily::Receipts, b"expected");
        let entry = StoredKvEntry {
            key_hex: hex::encode(b"unexpected"),
            value_hex: hex::encode(b"value"),
        };
        let encoded = serde_json::to_vec(&entry).expect("entry encoding must succeed");
        FileKvDb::write_atomic(&path, &encoded).expect("tampered write must succeed");

        let err = db
            .get(ColumnFamily::Receipts, b"expected")
            .expect_err("tampered entry must fail");

        match err {
            DataError::Integrity(message) => assert!(message.contains("stored key")),
            other => panic!("unexpected error: {other}"),
        }

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }

    #[test]
    fn empty_batch_is_a_noop() {
        let dir = unique_temp_dir("empty_batch");
        let db = FileKvDb::new(&dir).expect("db init must succeed");

        db.apply_batch(&[]).expect("empty batch must succeed");

        fs::remove_dir_all(dir).expect("temporary directory must be removable");
    }
}
