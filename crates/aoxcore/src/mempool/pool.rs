use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// Represents a validated transaction accepted into the mempool.
///
/// The structure intentionally stores:
/// - a stable transaction identifier used for duplicate suppression,
/// - the opaque transaction payload,
/// - the insertion timestamp for expiry enforcement.
///
/// This design keeps the mempool implementation generic while still exposing
/// the minimum metadata required for safe capacity accounting and lifecycle control.
#[derive(Debug, Clone)]
pub struct Transaction {
    id: [u8; 32],
    payload: Vec<u8>,
    inserted_at: Instant,
}

impl Transaction {
    /// Constructs a new transaction instance.
    ///
    /// # Security considerations
    /// The caller is responsible for ensuring that `id` is a canonical and collision-resistant
    /// identifier for `payload`. In production environments this should normally be the
    /// transaction hash produced by the system's canonical encoding rules.
    #[must_use]
    pub fn new(id: [u8; 32], payload: Vec<u8>) -> Self {
        Self {
            id,
            payload,
            inserted_at: Instant::now(),
        }
    }

    /// Returns the canonical transaction identifier.
    #[must_use]
    pub fn id(&self) -> &[u8; 32] {
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

    /// Returns the insertion timestamp.
    #[must_use]
    pub fn inserted_at(&self) -> Instant {
        self.inserted_at
    }
}

/// Configuration governing mempool resource boundaries and retention policy.
///
/// All fields are mandatory and explicit to avoid ambiguous runtime defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MempoolConfig {
    /// Maximum number of live transactions that may be retained at once.
    pub max_txs: usize,
    /// Maximum payload size allowed for a single transaction.
    pub max_tx_size: usize,
    /// Maximum total payload bytes retained across all live transactions.
    pub max_total_bytes: usize,
    /// Maximum retention duration for a transaction before it becomes collect-ineligible.
    pub tx_ttl: Duration,
}

impl MempoolConfig {
    /// Validates the configuration and returns an error if any safety invariant is violated.
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
///
/// This error model is intentionally strict so that the caller can
/// distinguish policy rejection from programming/configuration errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MempoolError {
    InvalidConfig(&'static str),
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

impl std::fmt::Display for MempoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MempoolError::InvalidConfig(msg) => write!(f, "invalid mempool config: {msg}"),
            MempoolError::EmptyTransaction => write!(f, "transaction payload must not be empty"),
            MempoolError::TransactionTooLarge { size, max_allowed } => write!(
                f,
                "transaction size {size} exceeds max allowed size {max_allowed}"
            ),
            MempoolError::DuplicateTransaction => {
                write!(f, "transaction already exists in mempool")
            }
            MempoolError::MempoolFull { max_txs } => {
                write!(f, "mempool has reached max transaction capacity {max_txs}")
            }
            MempoolError::TotalBytesExceeded {
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

/// Represents a transaction entry returned to the caller during collection.
///
/// Returning the full structure preserves deterministic metadata and avoids
/// hidden loss of context at the interface boundary.
#[derive(Debug, Clone)]
pub struct CollectedTransaction {
    pub id: [u8; 32],
    pub payload: Vec<u8>,
    pub inserted_at: Instant,
}

/// Production-oriented FIFO mempool with bounded resource consumption.
///
/// # Design notes
/// - FIFO ordering is intentionally preserved for deterministic collection.
/// - Duplicate suppression is implemented through an index map keyed by tx id.
/// - Resource bounds are enforced before insertion to prevent unbounded growth.
/// - Expired transactions are lazily evicted during public mutating operations.
///
/// This component is single-threaded by design. If concurrent access is required,
/// it should be wrapped by an external synchronization primitive at the integration layer.
#[derive(Debug)]
pub struct Mempool {
    config: MempoolConfig,
    queue: VecDeque<Transaction>,
    index: HashMap<[u8; 32], ()>,
    total_bytes: usize,
}

impl Mempool {
    /// Creates a new mempool instance after validating all configuration invariants.
    pub fn new(config: MempoolConfig) -> Result<Self, MempoolError> {
        let config = config.validate()?;

        Ok(Self {
            config,
            queue: VecDeque::new(),
            index: HashMap::new(),
            total_bytes: 0,
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
    pub fn contains(&self, id: &[u8; 32]) -> bool {
        self.index.contains_key(id)
    }

    /// Attempts to insert a new transaction into the mempool.
    ///
    /// # Rejection policy
    /// The transaction is rejected if any of the following conditions hold:
    /// - payload is empty,
    /// - payload exceeds the configured per-transaction limit,
    /// - transaction id already exists,
    /// - mempool transaction count limit would be exceeded,
    /// - mempool total byte limit would be exceeded.
    ///
    /// Expired entries are purged before capacity evaluation so that stale data
    /// does not artificially block fresh admissions.
    pub fn add_tx(&mut self, id: [u8; 32], payload: Vec<u8>) -> Result<(), MempoolError> {
        self.evict_expired();

        if payload.is_empty() {
            return Err(MempoolError::EmptyTransaction);
        }

        let tx_size = payload.len();

        if tx_size > self.config.max_tx_size {
            return Err(MempoolError::TransactionTooLarge {
                size: tx_size,
                max_allowed: self.config.max_tx_size,
            });
        }

        if self.index.contains_key(&id) {
            return Err(MempoolError::DuplicateTransaction);
        }

        if self.queue.len() >= self.config.max_txs {
            return Err(MempoolError::MempoolFull {
                max_txs: self.config.max_txs,
            });
        }

        if self.total_bytes.saturating_add(tx_size) > self.config.max_total_bytes {
            return Err(MempoolError::TotalBytesExceeded {
                current_bytes: self.total_bytes,
                tx_size,
                max_allowed: self.config.max_total_bytes,
            });
        }

        let tx = Transaction::new(id, payload);
        self.total_bytes += tx.size();
        self.index.insert(id, ());
        self.queue.push_back(tx);

        Ok(())
    }

    /// Collects up to `limit` non-expired transactions in FIFO order.
    ///
    /// Expired entries encountered at the front are discarded and not returned.
    /// A zero limit is treated as a valid no-op and returns an empty vector.
    pub fn collect(&mut self, limit: usize) -> Vec<CollectedTransaction> {
        self.evict_expired();

        if limit == 0 || self.queue.is_empty() {
            return Vec::new();
        }

        let take = limit.min(self.queue.len());
        let mut block_txs = Vec::with_capacity(take);

        for _ in 0..take {
            let Some(tx) = self.queue.pop_front() else {
                break;
            };

            self.total_bytes -= tx.size();
            self.index.remove(&tx.id);

            block_txs.push(CollectedTransaction {
                id: tx.id,
                payload: tx.payload,
                inserted_at: tx.inserted_at,
            });
        }

        block_txs
    }

    /// Removes a transaction by identifier if it exists.
    ///
    /// Returns true if a live entry was found and removed.
    /// This operation is O(n) due to FIFO queue preservation.
    pub fn remove_tx(&mut self, id: &[u8; 32]) -> bool {
        self.evict_expired();

        let Some(position) = self.queue.iter().position(|tx| &tx.id == id) else {
            return false;
        };

        let Some(tx) = self.queue.remove(position) else {
            return false;
        };

        self.total_bytes -= tx.size();
        self.index.remove(&tx.id);
        true
    }

    /// Returns the number of expired transactions removed during this call.
    ///
    /// This method can be invoked proactively by the integration layer to bound
    /// stale residency even during periods of low mempool activity.
    pub fn purge_expired(&mut self) -> usize {
        self.evict_expired()
    }

    /// Removes all entries from the mempool and resets accounting.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.index.clear();
        self.total_bytes = 0;
    }

    /// Returns a snapshot of transaction identifiers in current FIFO order.
    ///
    /// This is primarily useful for observability and deterministic testing.
    #[must_use]
    pub fn ids_in_order(&self) -> Vec<[u8; 32]> {
        self.queue.iter().map(|tx| tx.id).collect()
    }

    /// Evicts expired transactions from the head of the queue.
    ///
    /// Because FIFO order is enforced, once the head is found to be non-expired,
    /// all later entries are guaranteed to be newer and therefore also non-expired.
    fn evict_expired(&mut self) -> usize {
        let now = Instant::now();
        let mut removed = 0usize;

        while let Some(front) = self.queue.front() {
            if now.duration_since(front.inserted_at) < self.config.tx_ttl {
                break;
            }

            let tx = self
                .queue
                .pop_front()
                .expect("MEMPOOL: front element must exist after successful front() check");

            self.total_bytes -= tx.size();
            self.index.remove(&tx.id);
            removed += 1;
        }

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_id(byte: u8) -> [u8; 32] {
        [byte; 32]
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
    }

    #[test]
    fn rejects_empty_transaction() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        let result = mempool.add_tx(sample_id(1), vec![]);

        assert_eq!(result, Err(MempoolError::EmptyTransaction));
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
    }

    #[test]
    fn rejects_when_tx_capacity_is_reached() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![2])
            .expect("tx2 must be accepted");
        mempool
            .add_tx(sample_id(3), vec![3])
            .expect("tx3 must be accepted");

        let result = mempool
            .add_tx(sample_id(4), vec![4])
            .expect_err("capacity overflow must be rejected");

        assert_eq!(result, MempoolError::MempoolFull { max_txs: 3 });
    }

    #[test]
    fn rejects_when_total_bytes_would_be_exceeded() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![0u8; 16])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![0u8; 16])
            .expect("tx2 must be accepted");

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
    }

    #[test]
    fn collect_preserves_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![2])
            .expect("tx2 must be accepted");
        mempool
            .add_tx(sample_id(3), vec![3])
            .expect("tx3 must be accepted");

        let collected = mempool.collect(2);

        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].id, sample_id(1));
        assert_eq!(collected[1].id, sample_id(2));
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 1);
        assert!(mempool.contains(&sample_id(3)));
    }

    #[test]
    fn collect_with_zero_limit_is_no_op() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2, 3])
            .expect("tx must be accepted");

        let collected = mempool.collect(0);

        assert!(collected.is_empty());
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
    }

    #[test]
    fn remove_existing_transaction_updates_accounting() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![3, 4, 5])
            .expect("tx2 must be accepted");

        let removed = mempool.remove_tx(&sample_id(1));

        assert!(removed);
        assert_eq!(mempool.len(), 1);
        assert_eq!(mempool.total_bytes(), 3);
        assert!(!mempool.contains(&sample_id(1)));
        assert!(mempool.contains(&sample_id(2)));
    }

    #[test]
    fn clear_resets_state() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1, 2])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![3])
            .expect("tx2 must be accepted");

        mempool.clear();

        assert_eq!(mempool.len(), 0);
        assert_eq!(mempool.total_bytes(), 0);
        assert!(mempool.is_empty());
    }

    #[test]
    fn ids_snapshot_matches_fifo_order() {
        let mut mempool =
            Mempool::new(default_config()).expect("valid config must construct mempool");

        mempool
            .add_tx(sample_id(1), vec![1])
            .expect("tx1 must be accepted");
        mempool
            .add_tx(sample_id(2), vec![2])
            .expect("tx2 must be accepted");
        mempool
            .add_tx(sample_id(3), vec![3])
            .expect("tx3 must be accepted");

        let ids = mempool.ids_in_order();

        assert_eq!(ids, vec![sample_id(1), sample_id(2), sample_id(3)]);
    }
}
