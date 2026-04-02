//! Deterministic in-memory state with journaling support.

use std::collections::BTreeMap;

/// Error produced by state operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// Journal checkpoint identifier is invalid.
    InvalidCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JournalEntry {
    Write {
        key: Vec<u8>,
        previous: Option<Vec<u8>>,
    },
}

/// Deterministic key-value state and rollback journal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JournaledState {
    kv: BTreeMap<Vec<u8>, Vec<u8>>,
    journal: Vec<JournalEntry>,
}

impl JournaledState {
    /// Reads a key.
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.kv.get(key).map(Vec::as_slice)
    }

    /// Writes a key and records the previous value in the journal.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        let previous = self.kv.insert(key.clone(), value);
        self.journal.push(JournalEntry::Write { key, previous });
    }

    /// Opens a rollback checkpoint at the current journal length.
    pub fn checkpoint(&self) -> usize {
        self.journal.len()
    }

    /// Commits everything up to checkpoint by truncating stale journal entries.
    pub fn commit(&mut self, checkpoint: usize) -> Result<(), StateError> {
        if checkpoint > self.journal.len() {
            return Err(StateError::InvalidCheckpoint);
        }
        self.journal.drain(..checkpoint);
        Ok(())
    }

    /// Rolls back all writes since `checkpoint`.
    pub fn rollback(&mut self, checkpoint: usize) -> Result<(), StateError> {
        if checkpoint > self.journal.len() {
            return Err(StateError::InvalidCheckpoint);
        }

        while self.journal.len() > checkpoint {
            if let Some(JournalEntry::Write { key, previous }) = self.journal.pop() {
                match previous {
                    Some(old) => {
                        self.kv.insert(key, old);
                    }
                    None => {
                        self.kv.remove(&key);
                    }
                }
            }
        }

        Ok(())
    }

    /// Returns a deterministic state root preimage bytes (not cryptographic hash).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for (k, v) in &self.kv {
            out.extend_from_slice(&(k.len() as u64).to_le_bytes());
            out.extend_from_slice(k);
            out.extend_from_slice(&(v.len() as u64).to_le_bytes());
            out.extend_from_slice(v);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::JournaledState;

    #[test]
    fn rollback_restores_previous_values() {
        let mut state = JournaledState::default();
        state.put(b"a".to_vec(), b"1".to_vec());
        let cp = state.checkpoint();
        state.put(b"a".to_vec(), b"2".to_vec());
        state.put(b"b".to_vec(), b"3".to_vec());
        state.rollback(cp).expect("valid checkpoint");

        assert_eq!(state.get(b"a"), Some(&b"1"[..]));
        assert_eq!(state.get(b"b"), None);
    }
}
