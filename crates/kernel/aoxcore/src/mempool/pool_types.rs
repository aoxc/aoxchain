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

