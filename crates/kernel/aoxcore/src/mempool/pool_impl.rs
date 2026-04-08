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

