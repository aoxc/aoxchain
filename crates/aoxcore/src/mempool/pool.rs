// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC production-grade FIFO mempool.
//!
//! This module implements a bounded, deterministic, single-threaded FIFO
//! mempool with explicit admission controls, duplicate suppression, expiry
//! policy, and operator-facing telemetry snapshots.
//!
//! Design objectives:
//! - deterministic FIFO collection order,
//! - explicit and validated resource bounds,
//! - duplicate suppression by canonical transaction identifier,
//! - explicit expiry and eviction behavior,
//! - telemetry-friendly rejection and lifecycle statistics,
//! - panic-free accounting and state transitions.
//!
//! Security rationale:
//! The mempool is not a consensus object by itself, but weak admission policy,
//! imprecise accounting, or ambiguous eviction rules can still become an
//! operational denial-of-service surface. All relevant bounds are therefore
//! explicit, validated, and enforced before insertion.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::{Duration, Instant};

/// Canonical transaction identifier width in bytes.
pub const TX_ID_LEN: usize = 32;

/// Stable severity classification for mempool-level failures.
///
/// Intended for telemetry labels, dashboards, and incident routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MempoolErrorSeverity {
    Warning,
    Error,
    Critical,
}

impl MempoolErrorSeverity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "WARN",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Stable category classification for mempool-level failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MempoolErrorCategory {
    Config,
    Admission,
    Capacity,
    Integrity,
}

impl MempoolErrorCategory {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Config => "CONFIG",
            Self::Admission => "ADMISSION",
            Self::Capacity => "CAPACITY",
            Self::Integrity => "INTEGRITY",
        }
    }
}

/// Source channel by which a transaction entered the admission pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdmissionSource {
    Rpc,
    P2P,
    Internal,
    Recovery,
}

impl AdmissionSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rpc => "RPC",
            Self::P2P => "P2P",
            Self::Internal => "INTERNAL",
            Self::Recovery => "RECOVERY",
        }
    }
}

/// Priority label recorded at admission time.
///
/// Security rationale:
/// Priority is intentionally recorded for observability and future policy
/// evolution, but it does not currently override deterministic FIFO ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AdmissionPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl AdmissionPriority {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Normal => "NORMAL",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Admission metadata attached to a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionMeta {
    pub received_at: Instant,
    pub inserted_at: Instant,
    pub source: AdmissionSource,
    pub priority: AdmissionPriority,
}

impl AdmissionMeta {
    /// Returns default admission metadata using the current monotonic instant.
    #[must_use]
    pub fn now(source: AdmissionSource, priority: AdmissionPriority) -> Self {
        let now = Instant::now();
        Self {
            received_at: now,
            inserted_at: now,
            source,
            priority,
        }
    }
}

impl Default for AdmissionMeta {
    fn default() -> Self {
        Self::now(AdmissionSource::Rpc, AdmissionPriority::Normal)
    }
}

/// Canonical transaction retained by the mempool.
///
/// Security rationale:
/// The structure stores the minimum metadata required for duplicate
/// suppression, expiry enforcement, bounded resource accounting, admission
/// tracing, and future reporting surfaces.
#[derive(Debug, Clone)]
pub struct Transaction {
    id: [u8; TX_ID_LEN],
    payload: Vec<u8>,
    meta: AdmissionMeta,
}

impl Transaction {
    /// Constructs a transaction using default admission metadata.
    #[must_use]
    pub fn new(id: [u8; TX_ID_LEN], payload: Vec<u8>) -> Self {
        Self {
            id,
            payload,
            meta: AdmissionMeta::default(),
        }
    }

    /// Constructs a transaction with explicit admission metadata.
    #[must_use]
    pub fn new_with_meta(id: [u8; TX_ID_LEN], payload: Vec<u8>, meta: AdmissionMeta) -> Self {
        Self { id, payload, meta }
    }

    /// Returns the canonical transaction identifier.
    #[must_use]
    pub fn id(&self) -> &[u8; TX_ID_LEN] {
        &self.id
    }

    /// Returns the immutable transaction payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the payload length in bytes.
    #[must_use]
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    /// Returns admission metadata.
    #[must_use]
    pub fn meta(&self) -> AdmissionMeta {
        self.meta
    }

    /// Returns true if the identifier is all-zero.
    #[must_use]
    pub fn has_zero_id(&self) -> bool {
        self.id == [0u8; TX_ID_LEN]
    }
}

/// Configuration governing mempool resource boundaries and retention policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolConfig {
    /// Maximum number of live transactions retained at once.
    pub max_txs: usize,

    /// Maximum payload size allowed for a single transaction.
    pub max_tx_size: usize,

    /// Maximum total payload bytes retained across all live transactions.
    pub max_total_bytes: usize,

    /// Maximum retention duration for a transaction before it becomes expired.
    pub tx_ttl: Duration,
}

impl MempoolConfig {
    /// Validates configuration invariants and returns the validated config.
    pub fn validate(self) -> Result<Self, MempoolError> {
        if self.max_txs == 0 {
            return Err(MempoolError::InvalidConfig(
                "max_txs must be greater than zero",
            ));
        }

        if self.max_tx_size == 0 {
            return Err(MempoolError::InvalidConfig(
                "max_tx_size must be greater than zero",
            ));
        }

        if self.max_total_bytes == 0 {
            return Err(MempoolError::InvalidConfig(
                "max_total_bytes must be greater than zero",
            ));
        }

        if self.max_tx_size > self.max_total_bytes {
            return Err(MempoolError::InvalidConfig(
                "max_tx_size cannot exceed max_total_bytes",
            ));
        }

        if self.tx_ttl.is_zero() {
            return Err(MempoolError::InvalidConfig(
                "tx_ttl must be greater than zero",
            ));
        }

        Ok(self)
    }
}

/// Enumerates all mempool-level failures with explicit semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolError {
    InvalidConfig(&'static str),
    ZeroTransactionId,
    EmptyTransaction,
    TransactionTooLarge {
        size: usize,
        max_allowed: usize,
    },
    DuplicateTransaction,
    MempoolFull {
        max_txs: usize,
    },
    TotalBytesExceeded {
        current_bytes: usize,
        tx_size: usize,
        max_allowed: usize,
    },
}

impl MempoolError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "MEMPOOL_INVALID_CONFIG",
            Self::ZeroTransactionId => "MEMPOOL_ZERO_TRANSACTION_ID",
            Self::EmptyTransaction => "MEMPOOL_EMPTY_TRANSACTION",
            Self::TransactionTooLarge { .. } => "MEMPOOL_TRANSACTION_TOO_LARGE",
            Self::DuplicateTransaction => "MEMPOOL_DUPLICATE_TRANSACTION",
            Self::MempoolFull { .. } => "MEMPOOL_FULL",
            Self::TotalBytesExceeded { .. } => "MEMPOOL_TOTAL_BYTES_EXCEEDED",
        }
    }

    #[must_use]
    pub const fn category(&self) -> MempoolErrorCategory {
        match self {
            Self::InvalidConfig(_) => MempoolErrorCategory::Config,
            Self::ZeroTransactionId | Self::DuplicateTransaction => MempoolErrorCategory::Integrity,
            Self::EmptyTransaction | Self::TransactionTooLarge { .. } => {
                MempoolErrorCategory::Admission
            }
            Self::MempoolFull { .. } | Self::TotalBytesExceeded { .. } => {
                MempoolErrorCategory::Capacity
            }
        }
    }

    #[must_use]
    pub const fn severity(&self) -> MempoolErrorSeverity {
        match self {
            Self::InvalidConfig(_) => MempoolErrorSeverity::Critical,
            Self::ZeroTransactionId | Self::DuplicateTransaction => MempoolErrorSeverity::Error,
            Self::EmptyTransaction | Self::TransactionTooLarge { .. } => {
                MempoolErrorSeverity::Warning
            }
            Self::MempoolFull { .. } | Self::TotalBytesExceeded { .. } => {
                MempoolErrorSeverity::Warning
            }
        }
    }
}

impl fmt::Display for MempoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "invalid mempool config: {msg}"),
            Self::ZeroTransactionId => {
                write!(f, "transaction identifier must not be all-zero")
            }
            Self::EmptyTransaction => write!(f, "transaction payload must not be empty"),
            Self::TransactionTooLarge { size, max_allowed } => write!(
                f,
                "transaction size {size} exceeds max allowed size {max_allowed}"
            ),
            Self::DuplicateTransaction => {
                write!(f, "transaction already exists in mempool")
            }
            Self::MempoolFull { max_txs } => {
                write!(f, "mempool has reached max transaction capacity {max_txs}")
            }
            Self::TotalBytesExceeded {
                current_bytes,
                tx_size,
                max_allowed,
            } => write!(
                f,
                "adding transaction of size {tx_size} would exceed max total bytes {max_allowed}; current bytes {current_bytes}"
            ),
        }
    }
}

impl std::error::Error for MempoolError {}

/// Transaction record returned during collection.
///
/// Returning the full structure preserves deterministic metadata and avoids
/// hidden loss of context at the interface boundary.
#[derive(Debug, Clone)]
pub struct CollectedTransaction {
    pub id: [u8; TX_ID_LEN],
    pub payload: Vec<u8>,
    pub meta: AdmissionMeta,
}

/// Cumulative rejection counters by reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RejectionStats {
    pub zero_id: u64,
    pub empty: u64,
    pub oversized: u64,
    pub duplicate: u64,
    pub full: u64,
    pub bytes_exceeded: u64,
}

/// Cumulative lifecycle counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LifecycleStats {
    pub accepted: u64,
    pub expired_evictions: u64,
    pub explicit_removals: u64,
    pub collected: u64,
    pub clears: u64,
}

/// Per-source admission counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceStats {
    pub rpc: u64,
    pub p2p: u64,
    pub internal: u64,
    pub recovery: u64,
}

impl SourceStats {
    fn increment(&mut self, source: AdmissionSource) {
        match source {
            AdmissionSource::Rpc => self.rpc += 1,
            AdmissionSource::P2P => self.p2p += 1,
            AdmissionSource::Internal => self.internal += 1,
            AdmissionSource::Recovery => self.recovery += 1,
        }
    }
}

/// Lightweight mempool statistics snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolStats {
    pub len: usize,
    pub total_bytes: usize,
    pub max_txs: usize,
    pub max_tx_size: usize,
    pub max_total_bytes: usize,
    pub remaining_tx_capacity: usize,
    pub remaining_byte_capacity: usize,
    pub rejection_stats: RejectionStats,
    pub lifecycle_stats: LifecycleStats,
    pub accepted_by_source: SourceStats,
}

/// Production-oriented FIFO mempool with bounded resource consumption.
#[derive(Debug)]
pub struct Mempool {
    config: MempoolConfig,
    queue: VecDeque<Transaction>,
    index: HashMap<[u8; TX_ID_LEN], usize>,
    total_bytes: usize,
    rejection_stats: RejectionStats,
    lifecycle_stats: LifecycleStats,
    accepted_by_source: SourceStats,
}

impl Mempool {
    /// Creates a new mempool after validating all configuration invariants.
    pub fn new(config: MempoolConfig) -> Result<Self, MempoolError> {
        let config = config.validate()?;

        Ok(Self {
            config,
            queue: VecDeque::new(),
            index: HashMap::new(),
            total_bytes: 0,
            rejection_stats: RejectionStats::default(),
            lifecycle_stats: LifecycleStats::default(),
            accepted_by_source: SourceStats::default(),
        })
    }

    /// Returns the active configuration.
    #[must_use]
    pub const fn config(&self) -> MempoolConfig {
        self.config
    }

    /// Returns the number of live transactions currently retained.
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns true when the mempool contains no live transactions.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Returns the total retained payload bytes.
    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Returns true if the transaction identifier is currently present.
    #[must_use]
    pub fn contains(&self, id: &[u8; TX_ID_LEN]) -> bool {
        self.index.contains_key(id)
    }

    /// Returns the remaining transaction-capacity slots.
    #[must_use]
    pub fn remaining_tx_capacity(&self) -> usize {
        self.config.max_txs.saturating_sub(self.queue.len())
    }

    /// Returns the remaining aggregate byte capacity.
    #[must_use]
    pub fn remaining_byte_capacity(&self) -> usize {
        self.config.max_total_bytes.saturating_sub(self.total_bytes)
    }

    /// Returns cumulative rejection statistics.
    #[must_use]
    pub const fn rejection_stats(&self) -> RejectionStats {
        self.rejection_stats
    }

    /// Returns cumulative lifecycle statistics.
    #[must_use]
    pub const fn lifecycle_stats(&self) -> LifecycleStats {
        self.lifecycle_stats
    }

    /// Returns per-source accepted admission statistics.
    #[must_use]
    pub const fn accepted_by_source(&self) -> SourceStats {
        self.accepted_by_source
    }

    /// Returns a statistics snapshot for observability.
    #[must_use]
    pub fn stats(&self) -> MempoolStats {
        MempoolStats {
            len: self.queue.len(),
            total_bytes: self.total_bytes,
            max_txs: self.config.max_txs,
            max_tx_size: self.config.max_tx_size,
            max_total_bytes: self.config.max_total_bytes,
            remaining_tx_capacity: self.remaining_tx_capacity(),
            remaining_byte_capacity: self.remaining_byte_capacity(),
            rejection_stats: self.rejection_stats,
            lifecycle_stats: self.lifecycle_stats,
            accepted_by_source: self.accepted_by_source,
        }
    }

    /// Attempts to insert a new transaction using default admission metadata.
    pub fn add_tx(&mut self, id: [u8; TX_ID_LEN], payload: Vec<u8>) -> Result<(), MempoolError> {
        self.add_transaction(Transaction::new(id, payload))
    }

    /// Attempts to insert a transaction together with explicit admission metadata.
    pub fn add_tx_with_meta(
        &mut self,
        id: [u8; TX_ID_LEN],
        payload: Vec<u8>,
        meta: AdmissionMeta,
    ) -> Result<(), MempoolError> {
        self.add_transaction(Transaction::new_with_meta(id, payload, meta))
    }

    /// Attempts to insert a pre-built transaction into the mempool.
    ///
    /// Rejection policy:
    /// - payload must be non-empty,
    /// - identifier must not be all-zero,
    /// - payload must not exceed the configured per-transaction bound,
    /// - transaction must not already exist,
    /// - transaction-count bound must not be exceeded,
    /// - aggregate byte bound must not be exceeded.
    ///
    /// Expired entries are purged before capacity evaluation so stale data does
    /// not artificially block fresh admissions.
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), MempoolError> {
        self.evict_expired();

        if tx.has_zero_id() {
            self.rejection_stats.zero_id += 1;
            return Err(MempoolError::ZeroTransactionId);
        }

        if tx.payload.is_empty() {
            self.rejection_stats.empty += 1;
            return Err(MempoolError::EmptyTransaction);
        }

        let tx_size = tx.size();

        if tx_size > self.config.max_tx_size {
            self.rejection_stats.oversized += 1;
            return Err(MempoolError::TransactionTooLarge {
                size: tx_size,
                max_allowed: self.config.max_tx_size,
            });
        }

        if self.index.contains_key(&tx.id) {
            self.rejection_stats.duplicate += 1;
            return Err(MempoolError::DuplicateTransaction);
        }

        if self.queue.len() >= self.config.max_txs {
            self.rejection_stats.full += 1;
            return Err(MempoolError::MempoolFull {
                max_txs: self.config.max_txs,
            });
        }

        if self.total_bytes.saturating_add(tx_size) > self.config.max_total_bytes {
            self.rejection_stats.bytes_exceeded += 1;
            return Err(MempoolError::TotalBytesExceeded {
                current_bytes: self.total_bytes,
                tx_size,
                max_allowed: self.config.max_total_bytes,
            });
        }

        self.total_bytes += tx_size;
        self.accepted_by_source.increment(tx.meta.source);
        self.lifecycle_stats.accepted += 1;
        self.queue.push_back(tx);
        self.rebuild_index();

        Ok(())
    }

    /// Collects up to `limit` non-expired transactions in FIFO order.
    ///
    /// Expired entries encountered at the head are discarded and not returned.
    /// A zero limit is treated as a valid no-op and returns an empty vector.
    pub fn collect(&mut self, limit: usize) -> Vec<CollectedTransaction> {
        self.evict_expired();

        if limit == 0 || self.queue.is_empty() {
            return Vec::new();
        }

        let take = limit.min(self.queue.len());
        let mut collected = Vec::with_capacity(take);

        for _ in 0..take {
            let Some(tx) = self.queue.pop_front() else {
                break;
            };

            self.total_bytes -= tx.size();
            self.lifecycle_stats.collected += 1;

            collected.push(CollectedTransaction {
                id: tx.id,
                payload: tx.payload,
                meta: tx.meta,
            });
        }

        self.rebuild_index();
        collected
    }

    /// Removes a transaction by identifier if it exists.
    ///
    /// Returns true if a live entry was found and removed.
    /// This operation is O(n) because FIFO queue preservation is intentional.
    pub fn remove_tx(&mut self, id: &[u8; TX_ID_LEN]) -> bool {
        self.evict_expired();

        let Some(position) = self.queue.iter().position(|tx| &tx.id == id) else {
            return false;
        };

        let Some(tx) = self.queue.remove(position) else {
            return false;
        };

        self.total_bytes -= tx.size();
        self.lifecycle_stats.explicit_removals += 1;
        self.rebuild_index();
        true
    }

    /// Returns the number of expired transactions removed during this call.
    pub fn purge_expired(&mut self) -> usize {
        self.evict_expired()
    }

    /// Removes all live entries from the mempool and resets live accounting
    /// while preserving cumulative counters.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.index.clear();
        self.total_bytes = 0;
        self.lifecycle_stats.clears += 1;
    }

    /// Returns a snapshot of transaction identifiers in current FIFO order.
    #[must_use]
    pub fn ids_in_order(&self) -> Vec<[u8; TX_ID_LEN]> {
        self.queue.iter().map(|tx| tx.id).collect()
    }

    /// Returns the identifier of the current head transaction, if any.
    #[must_use]
    pub fn peek_oldest_id(&self) -> Option<[u8; TX_ID_LEN]> {
        self.queue.front().map(|tx| tx.id)
    }

    /// Returns the priority of the current head transaction, if any.
    #[must_use]
    pub fn peek_oldest_priority(&self) -> Option<AdmissionPriority> {
        self.queue.front().map(|tx| tx.meta.priority)
    }

    /// Evicts expired transactions from the head of the queue.
    ///
    /// Because FIFO order is preserved, once the head is non-expired all later
    /// entries are necessarily newer and therefore also non-expired.
    fn evict_expired(&mut self) -> usize {
        let now = Instant::now();
        let mut removed = 0usize;

        while let Some(front) = self.queue.front() {
            if now.duration_since(front.meta.inserted_at) < self.config.tx_ttl {
                break;
            }

            let Some(tx) = self.queue.pop_front() else {
                break;
            };

            self.total_bytes -= tx.size();
            removed += 1;
            self.lifecycle_stats.expired_evictions += 1;
        }

        if removed > 0 {
            self.rebuild_index();
        }

        removed
    }

    /// Rebuilds the identifier index from the current FIFO queue state.
    ///
    /// Operational rationale:
    /// The queue is the source of truth. Rebuilding the index avoids stale
    /// positions after collection, removal, or expiry.
    fn rebuild_index(&mut self) {
        self.index.clear();

        for (position, tx) in self.queue.iter().enumerate() {
            self.index.insert(tx.id, position);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id(byte: u8) -> [u8; TX_ID_LEN] {
        [byte; TX_ID_LEN]
    }

    fn default_config() -> MempoolConfig {
        MempoolConfig {
            max_txs: 3,
            max_tx_size: 16,
            max_total_bytes: 32,
            tx_ttl: Duration::from_secs(60),
        }
    }

    #[test]
    fn creates_valid_mempool() {
        let mempool = Mempool::new(default_config()).expect("valid config must construct mempool");
        assert_eq!(mempool.len(), 0);
        assert!(mempool.is_empty());
        assert_eq!(mempool.total_bytes(), 0);
    }

    #[test]
    fn rejects_invalid_config() {
        let result = Mempool::new(MempoolConfig {
            max_txs: 0,
            max_tx_size: 16,
            max_total_bytes: 32,
            tx_ttl: Duration::from_secs(60),
        });

        assert!(matches!(result, Err(MempoolError::InvalidConfig(_))));
    }

    #[test]
    fn accepts_valid_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect("valid transaction must be accepted");

        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
        assert!(mempool.contains(&sample_id(1)));
        assert_eq!(mempool.lifecycle_stats().accepted, 1);
    }

    #[test]
    fn rejects_zero_transaction_id() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx([0u8; TX_ID_LEN], vec![1]);

        assert_eq!(result, Err(MempoolError::ZeroTransactionId));
        assert_eq!(mempool.rejection_stats().zero_id, 1);
    }

    #[test]
    fn rejects_empty_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx(sample_id(1), vec![]);

        assert_eq!(result, Err(MempoolError::EmptyTransaction));
        assert_eq!(mempool.rejection_stats().empty, 1);
    }

    #[test]
    fn rejects_oversized_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx(sample_id(1), vec![0u8; 17]);

        assert_eq!(
            result,
            Err(MempoolError::TransactionTooLarge {
                size: 17,
                max_allowed: 16,
            })
        );
        assert_eq!(mempool.rejection_stats().oversized, 1);
    }

    #[test]
    fn rejects_duplicate_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect("first transaction must be accepted");
        let result = mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect_err("duplicate transaction must be rejected");

        assert_eq!(result, MempoolError::DuplicateTransaction);
        assert_eq!(mempool.rejection_stats().duplicate, 1);
    }

    #[test]
    fn rejects_when_tx_capacity_is_reached() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let result = mempool
            .add_tx(sample_id(4), vec![4])
            .expect_err("capacity overflow must be rejected");

        assert_eq!(result, MempoolError::MempoolFull { max_txs: 3 });
        assert_eq!(mempool.rejection_stats().full, 1);
    }

    #[test]
    fn rejects_when_total_bytes_would_be_exceeded() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![0u8; 16]).unwrap();
        mempool.add_tx(sample_id(2), vec![0u8; 16]).unwrap();

        let result = mempool
            .add_tx(sample_id(3), vec![1])
            .expect_err("total bytes overflow must be rejected");

        assert_eq!(
            result,
            MempoolError::TotalBytesExceeded {
                current_bytes: 32,
                tx_size: 1,
                max_allowed: 32,
            }
        );
        assert_eq!(mempool.rejection_stats().bytes_exceeded, 1);
    }

    #[test]
    fn collect_preserves_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let collected = mempool.collect(2);

        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].id, sample_id(1));
        assert_eq!(collected[1].id, sample_id(2));
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 1);
        assert!(mempool.contains(&sample_id(3)));
        assert_eq!(mempool.lifecycle_stats().collected, 2);
    }

    #[test]
    fn collect_with_zero_limit_is_no_op() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2, 3]).unwrap();

        let collected = mempool.collect(0);

        assert!(collected.is_empty());
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
    }

    #[test]
    fn remove_existing_transaction_updates_accounting() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2]).unwrap();
        mempool.add_tx(sample_id(2), vec![3, 4, 5]).unwrap();

        let removed = mempool.remove_tx(&sample_id(1));

        assert!(removed);
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
        assert!(!mempool.contains(&sample_id(1)));
        assert!(mempool.contains(&sample_id(2)));
        assert_eq!(mempool.lifecycle_stats().explicit_removals, 1);
    }

    #[test]
    fn clear_resets_live_state_and_preserves_counters() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2]).unwrap();
        mempool.add_tx(sample_id(2), vec![3]).unwrap();

        mempool.clear();

        assert_eq!(mempool.len(), 0);
        assert_eq!(mempool.total_bytes(), 0);
        assert!(mempool.is_empty());
        assert_eq!(mempool.lifecycle_stats().clears, 1);
    }

    #[test]
    fn ids_snapshot_matches_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1]).unwrap();
        mempool.add_tx(sample_id(2), vec![2]).unwrap();
        mempool.add_tx(sample_id(3), vec![3]).unwrap();

        let ids = mempool.ids_in_order();

        assert_eq!(ids, vec![sample_id(1), sample_id(2), sample_id(3)]);
    }

    #[test]
    fn stats_snapshot_matches_runtime_state() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool.add_tx(sample_id(1), vec![1, 2, 3]).unwrap();
        let stats = mempool.stats();

        assert_eq!(stats.len, 1);
        assert_eq!(stats.total_bytes, 3);
        assert_eq!(stats.max_txs, 3);
        assert_eq!(stats.max_tx_size, 16);
        assert_eq!(stats.max_total_bytes, 32);
        assert_eq!(stats.remaining_tx_capacity, 2);
        assert_eq!(stats.remaining_byte_capacity, 29);
        assert_eq!(stats.lifecycle_stats.accepted, 1);
    }

    #[test]
    fn explicit_metadata_is_preserved_on_collection() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let meta = AdmissionMeta::now(AdmissionSource::P2P, AdmissionPriority::High);

        mempool
            .add_tx_with_meta(sample_id(9), vec![1, 2, 3], meta)
            .unwrap();

        let collected = mempool.collect(1);
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0].meta.source, AdmissionSource::P2P);
        assert_eq!(collected[0].meta.priority, AdmissionPriority::High);
    }

    #[test]
    fn accepted_by_source_is_tracked() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx_with_meta(
                sample_id(1),
                vec![1],
                AdmissionMeta::now(AdmissionSource::Rpc, AdmissionPriority::Normal),
            )
            .unwrap();
        mempool
            .add_tx_with_meta(
                sample_id(2),
                vec![2],
                AdmissionMeta::now(AdmissionSource::P2P, AdmissionPriority::Normal),
            )
            .unwrap();

        let by_source = mempool.accepted_by_source();
        assert_eq!(by_source.rpc, 1);
        assert_eq!(by_source.p2p, 1);
    }
}
