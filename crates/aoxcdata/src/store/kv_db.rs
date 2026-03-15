use crate::store::column_families::ColumnFamily;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

type KvKey = (ColumnFamily, Vec<u8>);
type KvStore = BTreeMap<KvKey, Vec<u8>>;

/// Lightweight KV abstraction. In production this can be implemented by RocksDB/Sled.
pub trait KvDb: Send + Sync {
    fn put(&self, cf: ColumnFamily, key: Vec<u8>, value: Vec<u8>);
    fn get(&self, cf: ColumnFamily, key: &[u8]) -> Option<Vec<u8>>;
}

#[derive(Debug, Default, Clone)]
pub struct InMemoryKvDb {
    inner: Arc<RwLock<KvStore>>,
}

impl KvDb for InMemoryKvDb {
    fn put(&self, cf: ColumnFamily, key: Vec<u8>, value: Vec<u8>) {
        if let Ok(mut guard) = self.inner.write() {
            guard.insert((cf, key), value);
        }
    }

    fn get(&self, cf: ColumnFamily, key: &[u8]) -> Option<Vec<u8>> {
        self.inner
            .read()
            .ok()
            .and_then(|guard| guard.get(&(cf, key.to_vec())).cloned())
    }
}
