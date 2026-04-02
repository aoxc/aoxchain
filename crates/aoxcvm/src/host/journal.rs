//! Transactional host journal primitives for phase-1 settlement safety.

use std::collections::{BTreeMap, BTreeSet};

/// Deterministic lane identifier.
pub type LaneId = String;

/// Key namespace used by the host journal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StateScope {
    Shared,
    Lane(LaneId),
}

/// Canonical state key reference used for conflict detection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScopedKey {
    pub scope: StateScope,
    pub key: Vec<u8>,
}

/// Journal event emitted during execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEvent {
    pub scope: StateScope,
    pub topic: String,
    pub payload: Vec<u8>,
}

/// Persistent host state partitions.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostStatePartitions {
    pub shared: BTreeMap<Vec<u8>, Vec<u8>>,
    pub lane_local: BTreeMap<LaneId, BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl HostStatePartitions {
    /// Reads a value from a scope.
    pub fn get(&self, scope: &StateScope, key: &[u8]) -> Option<&[u8]> {
        match scope {
            StateScope::Shared => self.shared.get(key).map(Vec::as_slice),
            StateScope::Lane(lane) => self
                .lane_local
                .get(lane)
                .and_then(|partition| partition.get(key))
                .map(Vec::as_slice),
        }
    }

    fn put(&mut self, scope: &StateScope, key: Vec<u8>, value: Vec<u8>) {
        match scope {
            StateScope::Shared => {
                self.shared.insert(key, value);
            }
            StateScope::Lane(lane) => {
                self.lane_local
                    .entry(lane.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
    }

    fn delete(&mut self, scope: &StateScope, key: &[u8]) {
        match scope {
            StateScope::Shared => {
                self.shared.remove(key);
            }
            StateScope::Lane(lane) => {
                if let Some(partition) = self.lane_local.get_mut(lane) {
                    partition.remove(key);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum JournalOp {
    Put { scope: StateScope, key: Vec<u8> },
    Delete { scope: StateScope, key: Vec<u8> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JournalCheckpoint {
    id: usize,
    snapshot: HostStatePartitions,
    events_len: usize,
    op_log_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransactionContext {
    working: HostStatePartitions,
    events: Vec<JournalEvent>,
    op_log: Vec<JournalOp>,
    conflict_set: BTreeSet<ScopedKey>,
    checkpoints: Vec<JournalCheckpoint>,
    next_checkpoint_id: usize,
}

impl TransactionContext {
    fn new(base: &HostStatePartitions) -> Self {
        Self {
            working: base.clone(),
            events: Vec::new(),
            op_log: Vec::new(),
            conflict_set: BTreeSet::new(),
            checkpoints: Vec::new(),
            next_checkpoint_id: 0,
        }
    }

    fn record_conflict(&mut self, scope: &StateScope, key: &[u8]) {
        self.conflict_set.insert(ScopedKey {
            scope: scope.clone(),
            key: key.to_vec(),
        });
    }
}

/// Commit output containing deterministic mutation evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitSummary {
    pub write_set: BTreeSet<ScopedKey>,
    pub delete_set: BTreeSet<ScopedKey>,
    pub conflict_set: BTreeSet<ScopedKey>,
    pub events: Vec<JournalEvent>,
}

/// Errors produced by host journal operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalError {
    NoActiveTransaction,
    TransactionAlreadyActive,
    InvalidCheckpoint,
    CommitRejected(String),
}

/// Deterministic phase-1 host transaction journal.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostJournal {
    persistent: HostStatePartitions,
    tx: Option<TransactionContext>,
}

impl HostJournal {
    /// Creates a journal with empty persistent state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a journal from an existing persistent state.
    pub fn from_persistent(persistent: HostStatePartitions) -> Self {
        Self {
            persistent,
            tx: None,
        }
    }

    /// Returns persistent state snapshot.
    pub fn persistent(&self) -> &HostStatePartitions {
        &self.persistent
    }

    /// Returns true if a transaction is active.
    pub fn is_active(&self) -> bool {
        self.tx.is_some()
    }

    /// Begins a transaction.
    pub fn begin_transaction(&mut self) -> Result<(), JournalError> {
        if self.tx.is_some() {
            return Err(JournalError::TransactionAlreadyActive);
        }

        self.tx = Some(TransactionContext::new(&self.persistent));
        Ok(())
    }

    /// Creates a deterministic checkpoint inside the active transaction.
    pub fn checkpoint(&mut self) -> Result<usize, JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;

        let id = tx.next_checkpoint_id;
        tx.next_checkpoint_id += 1;
        tx.checkpoints.push(JournalCheckpoint {
            id,
            snapshot: tx.working.clone(),
            events_len: tx.events.len(),
            op_log_len: tx.op_log.len(),
        });

        Ok(id)
    }

    /// Writes a key/value into the active transaction state.
    pub fn put(
        &mut self,
        scope: StateScope,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;

        tx.working.put(&scope, key.clone(), value);
        tx.op_log.push(JournalOp::Put {
            scope: scope.clone(),
            key: key.clone(),
        });
        tx.record_conflict(&scope, &key);
        Ok(())
    }

    /// Deletes a key in the active transaction state.
    pub fn delete(&mut self, scope: StateScope, key: &[u8]) -> Result<(), JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;

        tx.working.delete(&scope, key);
        tx.op_log.push(JournalOp::Delete {
            scope: scope.clone(),
            key: key.to_vec(),
        });
        tx.record_conflict(&scope, key);
        Ok(())
    }

    /// Emits an execution event into the active transaction buffer.
    pub fn emit_event(&mut self, event: JournalEvent) -> Result<(), JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;
        tx.events.push(event);
        Ok(())
    }

    /// Reads from active transaction state when present, otherwise from persistent state.
    pub fn get(&self, scope: &StateScope, key: &[u8]) -> Option<&[u8]> {
        self.tx
            .as_ref()
            .map(|tx| tx.working.get(scope, key))
            .unwrap_or_else(|| self.persistent.get(scope, key))
    }

    /// Rolls back transaction state to checkpoint.
    pub fn rollback(&mut self, checkpoint_id: usize) -> Result<(), JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;

        let index = tx
            .checkpoints
            .iter()
            .position(|cp| cp.id == checkpoint_id)
            .ok_or(JournalError::InvalidCheckpoint)?;

        let checkpoint = tx.checkpoints[index].clone();
        tx.working = checkpoint.snapshot;
        tx.events.truncate(checkpoint.events_len);
        tx.op_log.truncate(checkpoint.op_log_len);

        tx.conflict_set.clear();
        let touched: Vec<(StateScope, Vec<u8>)> = tx
            .op_log
            .iter()
            .map(|op| match op {
                JournalOp::Put { scope, key } | JournalOp::Delete { scope, key } => {
                    (scope.clone(), key.clone())
                }
            })
            .collect();

        for (scope, key) in touched {
            tx.record_conflict(&scope, &key);
        }

        tx.checkpoints.truncate(index + 1);
        Ok(())
    }

    /// Merges checkpoint into parent transaction state by removing the checkpoint barrier.
    pub fn merge_checkpoint(&mut self, checkpoint_id: usize) -> Result<(), JournalError> {
        let tx = self.tx.as_mut().ok_or(JournalError::NoActiveTransaction)?;

        let index = tx
            .checkpoints
            .iter()
            .position(|cp| cp.id == checkpoint_id)
            .ok_or(JournalError::InvalidCheckpoint)?;

        tx.checkpoints.remove(index);
        Ok(())
    }

    /// Rolls back the whole transaction and clears active context.
    pub fn rollback_transaction(&mut self) -> Result<(), JournalError> {
        if self.tx.is_none() {
            return Err(JournalError::NoActiveTransaction);
        }
        self.tx = None;
        Ok(())
    }

    /// Returns conflicts intersecting an external conflict set.
    pub fn external_conflicts(
        &self,
        external: &BTreeSet<ScopedKey>,
    ) -> Result<BTreeSet<ScopedKey>, JournalError> {
        let tx = self.tx.as_ref().ok_or(JournalError::NoActiveTransaction)?;
        Ok(tx
            .conflict_set
            .intersection(external)
            .cloned()
            .collect::<BTreeSet<_>>())
    }

    /// Commits the active transaction into persistent state.
    pub fn commit(&mut self) -> Result<CommitSummary, JournalError> {
        self.commit_with_policy(|_| Ok(()))
    }

    /// Commits the active transaction with caller-provided validation.
    pub fn commit_with_policy<F>(&mut self, validate: F) -> Result<CommitSummary, JournalError>
    where
        F: FnOnce(&TransactionView<'_>) -> Result<(), String>,
    {
        let tx = self.tx.as_ref().ok_or(JournalError::NoActiveTransaction)?;

        let view = TransactionView {
            working: &tx.working,
            conflict_set: &tx.conflict_set,
            events: &tx.events,
        };

        if let Err(err) = validate(&view) {
            return Err(JournalError::CommitRejected(err));
        }

        let tx = self.tx.take().expect("checked active tx");

        let mut write_set = BTreeSet::new();
        let mut delete_set = BTreeSet::new();
        for op in &tx.op_log {
            match op {
                JournalOp::Put { scope, key } => {
                    write_set.insert(ScopedKey {
                        scope: scope.clone(),
                        key: key.clone(),
                    });
                }
                JournalOp::Delete { scope, key } => {
                    delete_set.insert(ScopedKey {
                        scope: scope.clone(),
                        key: key.clone(),
                    });
                }
            }
        }

        self.persistent = tx.working;

        Ok(CommitSummary {
            write_set,
            delete_set,
            conflict_set: tx.conflict_set,
            events: tx.events,
        })
    }
}

/// Read-only view over active transaction contents for policy checks.
pub struct TransactionView<'a> {
    pub working: &'a HostStatePartitions,
    pub conflict_set: &'a BTreeSet<ScopedKey>,
    pub events: &'a [JournalEvent],
}

#[cfg(test)]
mod tests {
    use super::{HostJournal, JournalError, JournalEvent, ScopedKey, StateScope};
    use std::collections::BTreeSet;

    #[test]
    fn nested_checkpoints_and_merge() {
        let mut journal = HostJournal::new();
        journal.begin_transaction().expect("tx begins");

        journal
            .put(StateScope::Shared, b"k1".to_vec(), b"v1".to_vec())
            .expect("write");
        let cp1 = journal.checkpoint().expect("cp1");

        journal
            .put(StateScope::Shared, b"k2".to_vec(), b"v2".to_vec())
            .expect("write");
        let cp2 = journal.checkpoint().expect("cp2");

        journal
            .put(StateScope::Shared, b"k3".to_vec(), b"v3".to_vec())
            .expect("write");
        journal.merge_checkpoint(cp2).expect("merge checkpoint");
        journal.rollback(cp1).expect("rollback to cp1");

        assert_eq!(journal.get(&StateScope::Shared, b"k1"), Some(&b"v1"[..]));
        assert_eq!(journal.get(&StateScope::Shared, b"k2"), None);
        assert_eq!(journal.get(&StateScope::Shared, b"k3"), None);
    }

    #[test]
    fn rollback_transaction_discards_all_mutations() {
        let mut journal = HostJournal::new();
        journal.begin_transaction().expect("tx begins");

        journal
            .put(
                StateScope::Lane("lane-a".into()),
                b"a".to_vec(),
                b"1".to_vec(),
            )
            .expect("write");
        journal
            .emit_event(JournalEvent {
                scope: StateScope::Lane("lane-a".into()),
                topic: "evt".into(),
                payload: b"payload".to_vec(),
            })
            .expect("event");

        journal
            .rollback_transaction()
            .expect("rollback whole transaction");
        assert_eq!(journal.get(&StateScope::Lane("lane-a".into()), b"a"), None);
        assert!(!journal.is_active());
    }

    #[test]
    fn detects_cross_lane_conflicts() {
        let mut journal = HostJournal::new();
        journal.begin_transaction().expect("tx begins");
        journal
            .put(
                StateScope::Lane("lane-1".into()),
                b"shared-key".to_vec(),
                b"x".to_vec(),
            )
            .expect("write");

        let mut external = BTreeSet::new();
        external.insert(ScopedKey {
            scope: StateScope::Lane("lane-1".into()),
            key: b"shared-key".to_vec(),
        });
        external.insert(ScopedKey {
            scope: StateScope::Lane("lane-2".into()),
            key: b"shared-key".to_vec(),
        });

        let conflicts = journal.external_conflicts(&external).expect("conflicts");
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts.contains(&ScopedKey {
            scope: StateScope::Lane("lane-1".into()),
            key: b"shared-key".to_vec(),
        }));
    }

    #[test]
    fn commit_is_atomic_when_policy_rejects() {
        let mut journal = HostJournal::new();
        journal.begin_transaction().expect("tx begins");
        journal
            .put(StateScope::Shared, b"safe".to_vec(), b"value".to_vec())
            .expect("write");

        let err = journal
            .commit_with_policy(|_| Err("policy-reject".to_string()))
            .expect_err("must reject");
        assert_eq!(
            err,
            JournalError::CommitRejected("policy-reject".to_string())
        );

        assert!(
            journal.is_active(),
            "transaction must stay active after reject"
        );
        assert_eq!(journal.persistent().get(&StateScope::Shared, b"safe"), None);

        let summary = journal.commit().expect("commit after policy fix");
        assert_eq!(
            journal.persistent().get(&StateScope::Shared, b"safe"),
            Some(&b"value"[..])
        );
        assert_eq!(summary.write_set.len(), 1);
    }
}
