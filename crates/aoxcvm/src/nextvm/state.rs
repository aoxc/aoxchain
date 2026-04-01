use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    StorageRead,
    StorageWrite,
    HostCall,
}

impl Capability {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Capability::StorageRead => "storage_read",
            Capability::StorageWrite => "storage_write",
            Capability::HostCall => "host_call",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StateStore {
    cells: HashMap<u64, u64>,
    caps: HashSet<Capability>,
}

impl StateStore {
    pub fn with_capabilities(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        let mut store = Self::default();
        for capability in capabilities {
            store.caps.insert(capability);
        }
        store
    }

    pub fn has_capability(&self, capability: Capability) -> bool {
        self.caps.contains(&capability)
    }

    pub fn set(&mut self, key: u64, value: u64) {
        self.cells.insert(key, value);
    }

    pub fn get(&self, key: u64) -> u64 {
        self.cells.get(&key).copied().unwrap_or_default()
    }
}
