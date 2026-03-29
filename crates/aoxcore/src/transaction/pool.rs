// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/transaction/src/pool.rs
//!
//! AOXC Transaction Pool.
//!
//! This module defines an in-memory pending transaction pool for validated
//! and signed AOXC transactions.
//!
//! Design objectives:
//! - Deterministic duplicate protection
//! - Sender/nonce conflict tracking
//! - Bounded admission policy
//! - Clean compatibility with canonical transaction hashing
//! - Clear extension path for fee-aware or reputation-aware scheduling
//! - Explicit internal index consistency checks

use core::fmt;
use std::collections::HashMap;

use super::{Transaction, TransactionError};

/// Canonical transaction identifier type.
pub type TransactionId = [u8; 32];

/// Canonical sender identity type.
pub type SenderId = [u8; 32];

/// Composite sender-nonce key used to prevent replay-style lane collisions
/// inside the pending pool.
type SenderNonceKey = (SenderId, u64);

/// Configuration for the transaction pool.
///
/// The default values are intentionally conservative but practical for an
/// in-memory baseline implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionPoolConfig {
    /// Maximum number of transactions permitted in the pool.
    pub max_transactions: usize,

    /// Maximum number of pending transactions permitted per sender.
    pub max_transactions_per_sender: usize,
}

impl Default for TransactionPoolConfig {
    fn default() -> Self {
        Self {
            max_transactions: 10_000,
            max_transactions_per_sender: 128,
        }
    }
}

impl TransactionPoolConfig {
    /// Validates the pool configuration under canonical pool rules.
    pub fn validate(self) -> Result<Self, TransactionPoolError> {
        if self.max_transactions == 0 {
            return Err(TransactionPoolError::InvalidConfig(
                InvalidPoolConfig::ZeroMaxTransactions,
            ));
        }

        if self.max_transactions_per_sender == 0 {
            return Err(TransactionPoolError::InvalidConfig(
                InvalidPoolConfig::ZeroMaxTransactionsPerSender,
            ));
        }

        if self.max_transactions_per_sender > self.max_transactions {
            return Err(TransactionPoolError::InvalidConfig(
                InvalidPoolConfig::PerSenderLimitExceedsGlobalLimit {
                    per_sender: self.max_transactions_per_sender,
                    global: self.max_transactions,
                },
            ));
        }

        Ok(self)
    }
}

/// Canonical invalid configuration reasons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidPoolConfig {
    ZeroMaxTransactions,
    ZeroMaxTransactionsPerSender,
    PerSenderLimitExceedsGlobalLimit { per_sender: usize, global: usize },
}

impl fmt::Display for InvalidPoolConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroMaxTransactions => {
                f.write_str("transaction pool max_transactions must be greater than zero")
            }
            Self::ZeroMaxTransactionsPerSender => f.write_str(
                "transaction pool max_transactions_per_sender must be greater than zero",
            ),
            Self::PerSenderLimitExceedsGlobalLimit { per_sender, global } => write!(
                f,
                "transaction pool max_transactions_per_sender ({}) exceeds max_transactions ({})",
                per_sender, global
            ),
        }
    }
}

impl std::error::Error for InvalidPoolConfig {}

/// Pool-domain error type.
///
/// This error type wraps transaction validation failures and pool-specific
/// admission or state-management failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionPoolError {
    /// The pool configuration is invalid.
    InvalidConfig(InvalidPoolConfig),

    /// The transaction failed structural or cryptographic validation before
    /// pool admission.
    TransactionRejected(TransactionError),

    /// A transaction with the same canonical transaction hash already exists.
    DuplicateTransactionId {
        /// The conflicting transaction identifier.
        tx_id: TransactionId,
    },

    /// A transaction from the same sender with the same nonce already occupies
    /// the pending lane.
    SenderNonceConflict {
        /// Sender public key.
        sender: SenderId,
        /// Conflicting nonce.
        nonce: u64,
        /// Existing transaction identifier.
        existing_tx_id: TransactionId,
    },

    /// The pool has reached its configured global capacity.
    PoolFull {
        /// Current transaction count.
        current: usize,
        /// Maximum configured transaction count.
        max: usize,
    },

    /// The sender has reached its configured per-sender capacity.
    SenderPoolLimitExceeded {
        /// Sender public key.
        sender: SenderId,
        /// Current transaction count for the sender.
        current: usize,
        /// Maximum configured count for the sender.
        max: usize,
    },

    /// The requested transaction identifier does not exist.
    TransactionNotFound {
        /// Missing transaction identifier.
        tx_id: TransactionId,
    },

    /// Internal indexes became inconsistent with the primary storage map.
    IndexInconsistency,
}

impl TransactionPoolError {
    /// Returns a stable symbolic code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "TX_POOL_INVALID_CONFIG",
            Self::TransactionRejected(_) => "TX_POOL_TRANSACTION_REJECTED",
            Self::DuplicateTransactionId { .. } => "TX_POOL_DUPLICATE_TRANSACTION_ID",
            Self::SenderNonceConflict { .. } => "TX_POOL_SENDER_NONCE_CONFLICT",
            Self::PoolFull { .. } => "TX_POOL_FULL",
            Self::SenderPoolLimitExceeded { .. } => "TX_POOL_SENDER_LIMIT_EXCEEDED",
            Self::TransactionNotFound { .. } => "TX_POOL_TRANSACTION_NOT_FOUND",
            Self::IndexInconsistency => "TX_POOL_INDEX_INCONSISTENCY",
        }
    }
}

impl fmt::Display for TransactionPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(err) => {
                write!(f, "transaction pool configuration is invalid ({})", err)
            }
            Self::TransactionRejected(err) => write!(
                f,
                "transaction pool admission failed: transaction validation rejected the candidate ({})",
                err
            ),
            Self::DuplicateTransactionId { tx_id } => write!(
                f,
                "transaction pool admission failed: duplicate transaction identifier detected ({:02x?})",
                tx_id
            ),
            Self::SenderNonceConflict {
                sender,
                nonce,
                existing_tx_id,
            } => write!(
                f,
                "transaction pool admission failed: sender/nonce lane conflict detected for sender {:02x?} and nonce {}; existing transaction id is {:02x?}",
                sender, nonce, existing_tx_id
            ),
            Self::PoolFull { current, max } => write!(
                f,
                "transaction pool admission failed: pool capacity reached (current {}, max {})",
                current, max
            ),
            Self::SenderPoolLimitExceeded {
                sender,
                current,
                max,
            } => write!(
                f,
                "transaction pool admission failed: sender {:02x?} reached per-sender capacity (current {}, max {})",
                sender, current, max
            ),
            Self::TransactionNotFound { tx_id } => write!(
                f,
                "transaction pool operation failed: transaction identifier not found ({:02x?})",
                tx_id
            ),
            Self::IndexInconsistency => {
                f.write_str("transaction pool internal indexes are inconsistent")
            }
        }
    }
}

impl std::error::Error for TransactionPoolError {}

impl From<TransactionError> for TransactionPoolError {
    fn from(value: TransactionError) -> Self {
        Self::TransactionRejected(value)
    }
}

/// In-memory pool of validated, pending transactions.
///
/// Internal indexing strategy:
/// - `pending`: canonical transaction id -> transaction
/// - `sender_nonces`: `(sender, nonce)` -> canonical transaction id
/// - `sender_counts`: sender -> number of pending transactions
///
/// This design provides:
/// - O(1)-ish lookup by transaction id
/// - O(1)-ish detection of sender/nonce conflicts
/// - O(1)-ish sender capacity tracking
#[derive(Debug)]
pub struct TransactionPool {
    config: TransactionPoolConfig,
    pending: HashMap<TransactionId, Transaction>,
    sender_nonces: HashMap<SenderNonceKey, TransactionId>,
    sender_counts: HashMap<SenderId, usize>,
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionPool {
    /// Creates an empty transaction pool using the default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(TransactionPoolConfig::default())
            .expect("default transaction pool configuration must be valid")
    }

    /// Creates an empty transaction pool using the provided configuration.
    pub fn with_config(config: TransactionPoolConfig) -> Result<Self, TransactionPoolError> {
        let config = config.validate()?;

        Ok(Self {
            config,
            pending: HashMap::new(),
            sender_nonces: HashMap::new(),
            sender_counts: HashMap::new(),
        })
    }

    /// Returns the active pool configuration.
    #[must_use]
    pub const fn config(&self) -> TransactionPoolConfig {
        self.config
    }

    /// Returns the number of pending transactions currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns `true` if the pool is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Returns `true` if the given transaction id exists in the pool.
    #[must_use]
    pub fn contains_tx_id(&self, tx_id: &TransactionId) -> bool {
        self.pending.contains_key(tx_id)
    }

    /// Returns `true` if a sender/nonce lane is currently occupied.
    #[must_use]
    pub fn contains_sender_nonce(&self, sender: &SenderId, nonce: u64) -> bool {
        self.sender_nonces.contains_key(&(*sender, nonce))
    }

    /// Returns the number of pending transactions currently associated with a sender.
    #[must_use]
    pub fn sender_transaction_count(&self, sender: &SenderId) -> usize {
        self.sender_counts.get(sender).copied().unwrap_or(0)
    }

    /// Returns a shared reference to a transaction by canonical transaction id.
    #[must_use]
    pub fn get(&self, tx_id: &TransactionId) -> Option<&Transaction> {
        self.pending.get(tx_id)
    }

    /// Returns the transaction identifier occupying the given sender/nonce lane.
    #[must_use]
    pub fn tx_id_by_sender_nonce(&self, sender: &SenderId, nonce: u64) -> Option<TransactionId> {
        self.sender_nonces.get(&(*sender, nonce)).copied()
    }

    /// Validates internal index consistency.
    ///
    /// This function is intended for tests, invariant assertions, recovery
    /// tooling, and debug or audit-oriented verification paths.
    pub fn validate(&self) -> Result<(), TransactionPoolError> {
        if self.pending.len() > self.config.max_transactions {
            return Err(TransactionPoolError::IndexInconsistency);
        }

        let mut derived_sender_counts: HashMap<SenderId, usize> = HashMap::new();

        for (tx_id, tx) in &self.pending {
            let sender_nonce = (tx.sender, tx.nonce);

            match self.sender_nonces.get(&sender_nonce) {
                Some(observed_tx_id) if observed_tx_id == tx_id => {}
                _ => return Err(TransactionPoolError::IndexInconsistency),
            }

            let count = derived_sender_counts.entry(tx.sender).or_insert(0);
            *count += 1;
        }

        if self.sender_nonces.len() != self.pending.len() {
            return Err(TransactionPoolError::IndexInconsistency);
        }

        for ((sender, _nonce), tx_id) in &self.sender_nonces {
            let tx = self
                .pending
                .get(tx_id)
                .ok_or(TransactionPoolError::IndexInconsistency)?;

            if tx.sender != *sender {
                return Err(TransactionPoolError::IndexInconsistency);
            }
        }

        if self.sender_counts.len() != derived_sender_counts.len() {
            return Err(TransactionPoolError::IndexInconsistency);
        }

        for (sender, derived_count) in derived_sender_counts {
            let observed = self
                .sender_counts
                .get(&sender)
                .copied()
                .ok_or(TransactionPoolError::IndexInconsistency)?;

            if observed != derived_count || observed > self.config.max_transactions_per_sender {
                return Err(TransactionPoolError::IndexInconsistency);
            }
        }

        Ok(())
    }

    /// Validates a transaction and inserts it into the pool.
    ///
    /// Admission policy:
    /// - Structural validation must succeed
    /// - Signature verification must succeed
    /// - Canonical signed transaction id must be unique
    /// - Sender/nonce lane must be unoccupied
    /// - Global pool capacity must not be exceeded
    /// - Per-sender capacity must not be exceeded
    ///
    /// The current implementation explicitly rejects replacements. A fee-aware
    /// or timestamp-aware replacement strategy can be added later without
    /// changing the core indexing model.
    pub fn add(&mut self, tx: Transaction) -> Result<TransactionId, TransactionPoolError> {
        tx.verify_signature()?;

        let tx_id = tx.try_tx_id()?;
        let sender_nonce = (tx.sender, tx.nonce);

        if self.pending.contains_key(&tx_id) {
            return Err(TransactionPoolError::DuplicateTransactionId { tx_id });
        }

        if let Some(existing_tx_id) = self.sender_nonces.get(&sender_nonce).copied() {
            return Err(TransactionPoolError::SenderNonceConflict {
                sender: tx.sender,
                nonce: tx.nonce,
                existing_tx_id,
            });
        }

        if self.pending.len() >= self.config.max_transactions {
            return Err(TransactionPoolError::PoolFull {
                current: self.pending.len(),
                max: self.config.max_transactions,
            });
        }

        let sender_count = self.sender_transaction_count(&tx.sender);
        if sender_count >= self.config.max_transactions_per_sender {
            return Err(TransactionPoolError::SenderPoolLimitExceeded {
                sender: tx.sender,
                current: sender_count,
                max: self.config.max_transactions_per_sender,
            });
        }

        self.pending.insert(tx_id, tx.clone());
        self.sender_nonces.insert(sender_nonce, tx_id);
        self.sender_counts.insert(tx.sender, sender_count + 1);

        debug_assert!(
            self.validate().is_ok(),
            "pool invariants must hold after add"
        );
        Ok(tx_id)
    }

    /// Removes a transaction from the pool by canonical transaction id.
    pub fn remove(&mut self, tx_id: &TransactionId) -> Result<Transaction, TransactionPoolError> {
        let tx = self
            .pending
            .remove(tx_id)
            .ok_or(TransactionPoolError::TransactionNotFound { tx_id: *tx_id })?;

        let sender_nonce = (tx.sender, tx.nonce);

        match self.sender_nonces.remove(&sender_nonce) {
            Some(observed_tx_id) if observed_tx_id == *tx_id => {}
            _ => return Err(TransactionPoolError::IndexInconsistency),
        }

        match self.sender_counts.get_mut(&tx.sender) {
            Some(count) if *count > 1 => {
                *count -= 1;
            }
            Some(count) if *count == 1 => {
                let _ = count;
                self.sender_counts.remove(&tx.sender);
            }
            _ => return Err(TransactionPoolError::IndexInconsistency),
        }

        debug_assert!(
            self.validate().is_ok(),
            "pool invariants must hold after remove"
        );
        Ok(tx)
    }

    /// Removes all pending transactions and clears all secondary indexes.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.sender_nonces.clear();
        self.sender_counts.clear();
    }

    /// Returns a deterministic snapshot of all pending transaction ids sorted in
    /// ascending lexicographic byte order.
    #[must_use]
    pub fn sorted_tx_ids(&self) -> Vec<TransactionId> {
        let mut ids: Vec<_> = self.pending.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// Returns a deterministic snapshot of pending transactions sorted by:
    /// 1. sender
    /// 2. nonce
    /// 3. signed transaction id
    ///
    /// This ordering is intentionally simple and deterministic. It provides a
    /// stable baseline for block assembly until a more advanced prioritization
    /// model is introduced.
    #[must_use]
    pub fn snapshot_ordered(&self) -> Vec<(TransactionId, &Transaction)> {
        let mut entries: Vec<_> = self
            .pending
            .iter()
            .map(|(tx_id, tx)| (*tx_id, tx))
            .collect();

        entries.sort_unstable_by(|(a_id, a_tx), (b_id, b_tx)| {
            a_tx.sender
                .cmp(&b_tx.sender)
                .then_with(|| a_tx.nonce.cmp(&b_tx.nonce))
                .then_with(|| a_id.cmp(b_id))
        });

        entries
    }

    /// Selects up to `max_count` transactions for block construction while
    /// respecting a maximum aggregate payload size.
    ///
    /// The selection order is deterministic and derived from [`snapshot_ordered`].
    #[must_use]
    pub fn select_for_block(
        &self,
        max_count: usize,
        max_total_payload_bytes: usize,
    ) -> Vec<(TransactionId, &Transaction)> {
        let mut selected = Vec::new();
        let mut accumulated_payload = 0usize;

        for (tx_id, tx) in self.snapshot_ordered() {
            if selected.len() >= max_count {
                break;
            }

            let payload_len = tx.payload_len();

            let next_payload = match accumulated_payload.checked_add(payload_len) {
                Some(value) => value,
                None => continue,
            };

            if next_payload > max_total_payload_bytes {
                continue;
            }

            accumulated_payload = next_payload;
            selected.push((tx_id, tx));
        }

        selected
    }

    /// Drains up to `max_count` transactions from the pool according to the
    /// deterministic selection order while respecting a maximum aggregate
    /// payload size.
    pub fn drain_for_block(
        &mut self,
        max_count: usize,
        max_total_payload_bytes: usize,
    ) -> Result<Vec<(TransactionId, Transaction)>, TransactionPoolError> {
        let selected_ids: Vec<_> = self
            .select_for_block(max_count, max_total_payload_bytes)
            .into_iter()
            .map(|(tx_id, _)| tx_id)
            .collect();

        let mut drained = Vec::with_capacity(selected_ids.len());

        for tx_id in selected_ids {
            let tx = self.remove(&tx_id)?;
            drained.push((tx_id, tx));
        }

        Ok(drained)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost};
    use crate::transaction::Transaction;
    use ed25519_dalek::{Signer, SigningKey};

    fn signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn signed_transaction(seed: u8, nonce: u64, payload: Vec<u8>) -> Transaction {
        let signing_key = signing_key(seed);
        let sender = signing_key.verifying_key().to_bytes();

        let unsigned = Transaction {
            sender,
            nonce,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload,
            signature: [0u8; 64],
        };

        let signature = signing_key.sign(&unsigned.signing_message()).to_bytes();

        Transaction {
            signature,
            ..unsigned
        }
    }

    #[test]
    fn default_config_is_valid() {
        let config = TransactionPoolConfig::default();
        assert_eq!(config.validate(), Ok(config));
    }

    #[test]
    fn invalid_config_is_rejected() {
        let result = TransactionPool::with_config(TransactionPoolConfig {
            max_transactions: 0,
            max_transactions_per_sender: 1,
        });

        assert!(matches!(
            result,
            Err(TransactionPoolError::InvalidConfig(
                InvalidPoolConfig::ZeroMaxTransactions
            ))
        ));
    }

    #[test]
    fn pool_accepts_valid_transaction() {
        let mut pool = TransactionPool::new();
        let tx = signed_transaction(1, 1, vec![1, 2, 3]);

        let tx_id = pool.add(tx).expect("valid transaction must be admitted");

        assert_eq!(pool.len(), 1);
        assert!(pool.contains_tx_id(&tx_id));
        assert!(pool.validate().is_ok());
    }

    #[test]
    fn pool_rejects_duplicate_transaction_id() {
        let mut pool = TransactionPool::new();
        let tx = signed_transaction(1, 1, vec![1, 2, 3]);
        let tx_clone = tx.clone();

        let first_id = pool.add(tx).expect("first transaction must be admitted");
        let result = pool.add(tx_clone);

        assert_eq!(
            result,
            Err(TransactionPoolError::DuplicateTransactionId { tx_id: first_id })
        );
    }

    #[test]
    fn pool_rejects_sender_nonce_conflict() {
        let mut pool = TransactionPool::new();

        let tx_a = signed_transaction(1, 7, vec![1, 2, 3]);
        let tx_b = signed_transaction(1, 7, vec![9, 9, 9]);

        let existing_tx_id = pool.add(tx_a).expect("first transaction must be admitted");

        let result = pool.add(tx_b);

        match result {
            Err(TransactionPoolError::SenderNonceConflict {
                sender: _,
                nonce,
                existing_tx_id: observed,
            }) => {
                assert_eq!(nonce, 7);
                assert_eq!(observed, existing_tx_id);
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn pool_rejects_when_global_capacity_is_reached() {
        let config = TransactionPoolConfig {
            max_transactions: 1,
            max_transactions_per_sender: 1,
        };

        let mut pool = TransactionPool::with_config(config).expect("config must be valid");
        pool.add(signed_transaction(1, 1, vec![1]))
            .expect("first transaction must be admitted");

        let result = pool.add(signed_transaction(2, 1, vec![2]));
        assert_eq!(
            result,
            Err(TransactionPoolError::PoolFull { current: 1, max: 1 })
        );
    }

    #[test]
    fn pool_rejects_when_sender_capacity_is_reached() {
        let config = TransactionPoolConfig {
            max_transactions: 16,
            max_transactions_per_sender: 1,
        };

        let mut pool = TransactionPool::with_config(config).expect("config must be valid");
        let sender_seed = 9;

        pool.add(signed_transaction(sender_seed, 1, vec![1]))
            .expect("first transaction must be admitted");

        let result = pool.add(signed_transaction(sender_seed, 2, vec![2]));

        match result {
            Err(TransactionPoolError::SenderPoolLimitExceeded {
                sender: _,
                current,
                max,
            }) => {
                assert_eq!(current, 1);
                assert_eq!(max, 1);
            }
            other => panic!("unexpected result: {:?}", other),
        }
    }

    #[test]
    fn pool_remove_clears_secondary_indexes() {
        let mut pool = TransactionPool::new();
        let tx = signed_transaction(2, 9, vec![4, 5, 6]);

        let tx_id = pool.add(tx).expect("transaction must be admitted");
        let removed = pool.remove(&tx_id).expect("transaction must be removable");

        assert_eq!(removed.nonce, 9);
        assert!(pool.is_empty());
        assert!(!pool.contains_sender_nonce(&removed.sender, removed.nonce));
        assert_eq!(pool.sender_transaction_count(&removed.sender), 0);
        assert!(pool.validate().is_ok());
    }

    #[test]
    fn selection_is_bounded_by_count_and_payload() {
        let mut pool = TransactionPool::new();

        pool.add(signed_transaction(1, 1, vec![1, 2, 3]))
            .expect("tx1 must be admitted");
        pool.add(signed_transaction(2, 1, vec![4, 5, 6]))
            .expect("tx2 must be admitted");
        pool.add(signed_transaction(3, 1, vec![7, 8, 9]))
            .expect("tx3 must be admitted");

        let selected = pool.select_for_block(2, 6);
        assert_eq!(selected.len(), 2);
        assert_eq!(
            selected
                .iter()
                .map(|(_, tx)| tx.payload_len())
                .sum::<usize>(),
            6
        );
    }

    #[test]
    fn drain_for_block_removes_selected_transactions() {
        let mut pool = TransactionPool::new();

        pool.add(signed_transaction(1, 1, vec![1]))
            .expect("tx1 must be admitted");
        pool.add(signed_transaction(2, 1, vec![2]))
            .expect("tx2 must be admitted");

        let drained = pool.drain_for_block(1, 1024).expect("drain must succeed");

        assert_eq!(drained.len(), 1);
        assert_eq!(pool.len(), 1);
        assert!(pool.validate().is_ok());
    }
}
