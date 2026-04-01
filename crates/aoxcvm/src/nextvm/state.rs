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
    journal: Vec<HashMap<u64, Option<u64>>>,
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
        if let Some(layer) = self.journal.last_mut() {
            layer.entry(key).or_insert_with(|| self.cells.get(&key).copied());
        }
        self.cells.insert(key, value);
    }

    pub fn get(&self, key: u64) -> u64 {
        self.cells.get(&key).copied().unwrap_or_default()
    }

    pub fn checkpoint(&mut self) {
        self.journal.push(HashMap::new());
    }

    pub fn commit(&mut self) {
        if self.journal.is_empty() {
            return;
        }
        let current = self.journal.pop().expect("journal checked as non-empty");
        if let Some(parent) = self.journal.last_mut() {
            for (key, old_value) in current {
                parent.entry(key).or_insert(old_value);
            }
        }
    }

    pub fn rollback(&mut self) {
        let Some(current) = self.journal.pop() else {
            return;
        };

        for (key, old_value) in current {
            match old_value {
                Some(value) => {
                    self.cells.insert(key, value);
                }
                None => {
                    self.cells.remove(&key);
                }
            }
        }
    }
}
