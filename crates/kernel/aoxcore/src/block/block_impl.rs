impl Block {
    /// Creates a validated active block using the current system timestamp.
    pub fn new_active(
        height: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            state_root,
            producer,
            BlockType::Active,
            tasks,
        )
    }

    /// Validates the block against a node key bundle by requiring the
    /// consensus-role public key to match the producer field.
    pub fn validate_with_key_bundle(
        &self,
        bundle: &crate::identity::key_bundle::NodeKeyBundleV1,
    ) -> Result<(), BlockError> {
        let consensus_key = bundle
            .public_key_bytes_for_role(crate::identity::key_bundle::NodeKeyRole::Consensus)
            .map_err(|_| BlockError::InvalidProducer)?;

        if self.header.producer != consensus_key {
            return Err(BlockError::InvalidProducer);
        }

        self.validate()
    }

    /// Creates a validated active block with an explicit timestamp.
    pub fn new_active_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            state_root,
            producer,
            BlockType::Active,
            tasks,
        )
    }

    /// Creates a validated heartbeat block using the current system timestamp.
    pub fn new_heartbeat(
        height: u64,
        prev_hash: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            ZERO_STATE_ROOT,
            producer,
            BlockType::Heartbeat,
            Vec::new(),
        )
    }

    /// Creates a validated heartbeat block with an explicit timestamp.
    pub fn new_heartbeat_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            ZERO_STATE_ROOT,
            producer,
            BlockType::Heartbeat,
            Vec::new(),
        )
    }

    /// Creates a validated epoch-prune block using the current system timestamp.
    pub fn new_epoch_prune(
        height: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            state_root,
            producer,
            BlockType::EpochPrune,
            Vec::new(),
        )
    }

    /// Creates a validated epoch-prune block with an explicit timestamp.
    pub fn new_epoch_prune_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            state_root,
            producer,
            BlockType::EpochPrune,
            Vec::new(),
        )
    }

    /// Builds a block and validates all domain invariants before returning it.
    fn build(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        block_type: BlockType,
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        let strict_profile = crate::protocol::quantum::QuantumKernelProfile::strict_default();
        let block = Self {
            header: BlockHeader {
                height,
                timestamp,
                prev_hash,
                state_root,
                producer,
                quantum_signature_scheme: crate::protocol::quantum::SignatureScheme::MlDsa65,
                quantum_header_proof: vec![0x01],
                block_type,
            },
            tasks,
        };

        block.validate()?;
        Ok(block)
    }

    /// Validates block-level invariants.
    ///
    /// Validation policy:
    /// - header semantics are validated first,
    /// - task count is globally bounded,
    /// - active blocks must contain tasks and enforce uniqueness and payload bounds,
    /// - heartbeat and epoch-prune blocks must not contain tasks,
    /// - heartbeat blocks must use the canonical zero state root.
    pub fn validate(&self) -> Result<(), BlockError> {
        self.validate_header_semantics()?;

        if self.tasks.len() > MAX_TASKS_PER_BLOCK {
            return Err(BlockError::TooManyTasks {
                count: self.tasks.len(),
                max: MAX_TASKS_PER_BLOCK,
            });
        }

        match self.header.block_type {
            BlockType::Active => self.validate_active_block_policy()?,
            BlockType::Heartbeat => self.validate_heartbeat_block_policy()?,
            BlockType::EpochPrune => self.validate_epoch_prune_block_policy()?,
        }

        Ok(())
    }

    /// Validates direct parent linkage against an expected parent block.
    ///
    /// Validation contract:
    /// - both blocks must independently validate,
    /// - child height must equal parent height + 1,
    /// - child previous hash must equal parent header hash,
    /// - child timestamp must not precede the parent timestamp.
    pub fn validate_parent_link(&self, parent: &Block) -> Result<(), BlockError> {
        self.validate()?;
        parent.validate()?;

        if self.header.height != parent.header.height.saturating_add(1) {
            return Err(BlockError::InvalidBlockHeight);
        }

        let expected_prev_hash = parent.header_hash();
        if self.header.prev_hash != expected_prev_hash {
            return Err(BlockError::InvalidPreviousHash);
        }

        if self.header.timestamp < parent.header.timestamp {
            return Err(BlockError::InvalidTimestamp);
        }

        Ok(())
    }

    /// Returns the task count.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns the aggregated payload size in bytes using saturating arithmetic.
    ///
    /// Operational rationale:
    /// This helper is intended for reporting and dashboards. Consensus-critical
    /// validation uses checked arithmetic in the validation path.
    #[must_use]
    pub fn total_payload_bytes(&self) -> usize {
        let mut total = 0usize;

        for task in &self.tasks {
            total = total.saturating_add(task.payload_len());
        }

        total
    }

    /// Returns `true` if the block is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.header.block_type == BlockType::Active
    }

    /// Returns `true` if the block is a heartbeat block.
    #[must_use]
    pub fn is_heartbeat(&self) -> bool {
        self.header.block_type == BlockType::Heartbeat
    }

    /// Returns `true` if the block is an epoch-prune block.
    #[must_use]
    pub fn is_epoch_prune(&self) -> bool {
        self.header.block_type == BlockType::EpochPrune
    }

    /// Returns the compact numeric block-type code.
    #[must_use]
    pub fn block_type_code(&self) -> u8 {
        self.header.block_type.code()
    }

    /// Returns the canonical header hash.
    #[must_use]
    pub fn header_hash(&self) -> [u8; 32] {
        hash::hash_header(&self.header)
    }

    /// Returns the canonical task-root commitment.
    pub fn task_root(&self) -> Result<[u8; 32], BlockError> {
        hash::calculate_task_root(&self.tasks)
    }

    /// Returns the canonical task-root commitment.
    pub fn try_task_root(&self) -> Result<[u8; 32], BlockError> {
        self.task_root()
    }

    /// Validates the block and returns a serializable operator-friendly report.
    #[must_use]
    pub fn validate_with_report(&self) -> BlockValidationReport {
        build_block_validation_report(self)
    }

    /// Returns `true` if the block contains duplicate task identifiers.
    ///
    /// This helper is exposed for production callers that want a quick
    /// diagnostic signal without changing the current `BlockError` contract.
    #[must_use]
    pub fn has_duplicate_task_ids(&self) -> bool {
        let mut seen: HashSet<[u8; 32]> = HashSet::with_capacity(self.tasks.len());

        for task in &self.tasks {
            if !seen.insert(task.task_id) {
                return true;
            }
        }

        false
    }

    /// Returns `true` when the block contains no tasks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    fn validate_header_semantics(&self) -> Result<(), BlockError> {
        if self.header.timestamp == 0 {
            return Err(BlockError::InvalidTimestamp);
        }

        if self.header.producer == ZERO_HASH {
            return Err(BlockError::InvalidProducer);
        }

        if self.header.quantum_signature_scheme != crate::protocol::quantum::SignatureScheme::MlDsa65
        {
            return Err(BlockError::InvalidProducer);
        }

        if self.header.quantum_header_proof.is_empty()
            || self.header.quantum_header_proof.len() > MAX_QUANTUM_HEADER_PROOF_BYTES
        {
            return Err(BlockError::InvalidProducer);
        }

        Ok(())
    }

    fn validate_active_block_policy(&self) -> Result<(), BlockError> {
        if self.tasks.is_empty() {
            return Err(BlockError::ActiveBlockRequiresTasks);
        }

        let mut seen_ids: HashSet<[u8; 32]> = HashSet::with_capacity(self.tasks.len());
        let mut total_payload = 0usize;

        for task in &self.tasks {
            task.validate()?;

            if !seen_ids.insert(task.task_id) {
                return Err(BlockError::DuplicateTaskId);
            }

            total_payload = total_payload
                .checked_add(task.payload_len())
                .ok_or(BlockError::LengthOverflow)?;
        }

        if total_payload > MAX_BLOCK_PAYLOAD_BYTES {
            return Err(BlockError::TotalPayloadTooLarge {
                size: total_payload,
                max: MAX_BLOCK_PAYLOAD_BYTES,
            });
        }

        Ok(())
    }

    fn validate_heartbeat_block_policy(&self) -> Result<(), BlockError> {
        if !self.tasks.is_empty() {
            return Err(BlockError::HeartbeatBlockMustNotContainTasks);
        }

        if self.header.state_root != ZERO_STATE_ROOT {
            return Err(BlockError::HeartbeatBlockMustUseZeroStateRoot);
        }

        Ok(())
    }

    fn validate_epoch_prune_block_policy(&self) -> Result<(), BlockError> {
        if !self.tasks.is_empty() {
            return Err(BlockError::EpochPruneBlockMustNotContainTasks);
        }

        Ok(())
    }
}

/// Returns the current Unix timestamp in seconds.
fn current_time() -> Result<u64, BlockError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BlockError::InvalidSystemTime)?;

    Ok(duration.as_secs())
}
