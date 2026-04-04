/// Canonical gas unit for execution accounting.
pub type Gas = u64;

pub(crate) const DOMAIN_EXEC_PAYLOAD_V1: &[u8] = b"AOXC_EXEC_PAYLOAD_V1";
pub(crate) const DOMAIN_EXEC_RECEIPT_V1: &[u8] = b"AOXC_EXEC_RECEIPT_V1";
pub(crate) const DOMAIN_EXEC_STATE_V1: &[u8] = b"AOXC_EXEC_STATE_V1";
pub(crate) const DOMAIN_EXEC_TRACE_V1: &[u8] = b"AOXC_EXEC_TRACE_V1";
pub(crate) const DOMAIN_EXEC_BLOCK_V1: &[u8] = b"AOXC_EXEC_BLOCK_V1";
pub(crate) const DOMAIN_EXEC_CONFIG_V1: &[u8] = b"AOXC_EXEC_CONFIG_V1";
pub(crate) const MAX_ERROR_MESSAGE_LEN: usize = 160;

/// Deterministic execution-lane policy enforced by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePolicy {
    pub lane_id: String,
    pub enabled: bool,
    pub base_gas: Gas,
    pub gas_per_byte: Gas,
    pub max_payload_bytes: usize,
    pub max_gas_per_tx: Gas,
    pub max_sender_txs_per_block: usize,
}

impl LanePolicy {
    #[must_use]
    pub fn new(
        lane_id: impl Into<String>,
        base_gas: Gas,
        gas_per_byte: Gas,
        max_payload_bytes: usize,
        max_gas_per_tx: Gas,
        max_sender_txs_per_block: usize,
    ) -> Self {
        Self {
            lane_id: lane_id.into(),
            enabled: true,
            base_gas,
            gas_per_byte,
            max_payload_bytes,
            max_gas_per_tx,
            max_sender_txs_per_block,
        }
    }

    pub fn validate(&self) -> Result<(), ExecutionError> {
        if self.lane_id.trim().is_empty() {
            return Err(ExecutionError::InvalidPolicy("lane_id must not be empty"));
        }
        if !self
            .lane_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(ExecutionError::InvalidPolicy(
                "lane_id contains invalid characters",
            ));
        }
        if self.base_gas == 0 {
            return Err(ExecutionError::InvalidPolicy(
                "base_gas must be greater than zero",
            ));
        }
        if self.max_payload_bytes == 0 || self.max_gas_per_tx == 0 {
            return Err(ExecutionError::InvalidPolicy(
                "max_payload_bytes and max_gas_per_tx must be greater than zero",
            ));
        }
        if self.max_sender_txs_per_block == 0 {
            return Err(ExecutionError::InvalidPolicy(
                "max_sender_txs_per_block must be greater than zero",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneRegistryPolicy {
    pub policy_id: String,
    pub policy_version: u32,
    pub activation_height: u64,
    pub checksum: [u8; 32],
    pub governance_approval_ref: String,
    pub policy: LanePolicy,
}

impl LaneRegistryPolicy {
    #[must_use]
    pub fn new(
        policy_id: impl Into<String>,
        policy_version: u32,
        activation_height: u64,
        governance_approval_ref: impl Into<String>,
        policy: LanePolicy,
    ) -> Self {
        let policy_id = policy_id.into();
        let governance_approval_ref = governance_approval_ref.into();
        let checksum = crate::engine::hash_struct(
            DOMAIN_EXEC_CONFIG_V1,
            &(
                &policy_id,
                policy_version,
                activation_height,
                &governance_approval_ref,
                &policy,
            ),
        );
        Self {
            policy_id,
            policy_version,
            activation_height,
            checksum,
            governance_approval_ref,
            policy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LaneRegistry {
    policies: BTreeMap<String, Vec<LaneRegistryPolicy>>,
}

impl LaneRegistry {
    #[must_use]
    pub fn new(entries: impl IntoIterator<Item = LaneRegistryPolicy>) -> Self {
        let mut policies: BTreeMap<String, Vec<LaneRegistryPolicy>> = BTreeMap::new();
        for entry in entries {
            policies
                .entry(entry.policy.lane_id.clone())
                .or_default()
                .push(entry);
        }
        for versions in policies.values_mut() {
            versions.sort_by_key(|entry| (entry.activation_height, entry.policy_version));
        }
        Self { policies }
    }

    pub fn resolve(
        &self,
        lane_id: &str,
        block_height: u64,
    ) -> Result<Option<&LanePolicy>, ExecutionError> {
        let Some(entries) = self.policies.get(lane_id) else {
            return Ok(None);
        };
        let mut active: Option<&LaneRegistryPolicy> = None;
        for entry in entries {
            crate::engine::validate_registry_checksum(entry)?;
            if entry.activation_height <= block_height {
                active = Some(entry);
            }
        }
        Ok(active.map(|entry| &entry.policy))
    }

    #[must_use]
    pub fn policy_versions(&self, block_height: u64) -> BTreeMap<String, u32> {
        self.policies
            .iter()
            .filter_map(|(lane_id, entries)| {
                entries
                    .iter()
                    .filter(|entry| entry.activation_height <= block_height)
                    .max_by_key(|entry| (entry.activation_height, entry.policy_version))
                    .map(|entry| (lane_id.clone(), entry.policy_version))
            })
            .collect()
    }
}

/// Execution errors that should reject the batch before receipt generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidContext(&'static str),
    InvalidPolicy(&'static str),
    DuplicateTransaction([u8; 32]),
    DuplicateSenderNonce {
        sender: [u8; 32],
        nonce: u64,
    },
    ArithmeticOverflow,
    SerializationFailure(&'static str),
    ConfigChecksumMismatch {
        lane_id: String,
        policy_version: u32,
    },
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext(reason) => write!(f, "invalid execution context: {reason}"),
            Self::InvalidPolicy(reason) => write!(f, "invalid lane policy: {reason}"),
            Self::DuplicateTransaction(tx_hash) => {
                write!(
                    f,
                    "duplicate transaction in batch: {}",
                    hex::encode(tx_hash)
                )
            }
            Self::DuplicateSenderNonce { sender, nonce } => write!(
                f,
                "duplicate sender/nonce in batch: sender={} nonce={nonce}",
                hex::encode(sender)
            ),
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow in execution accounting"),
            Self::SerializationFailure(reason) => write!(f, "serialization failure: {reason}"),
            Self::ConfigChecksumMismatch {
                lane_id,
                policy_version,
            } => write!(
                f,
                "lane policy checksum mismatch for lane '{lane_id}' version {policy_version}"
            ),
        }
    }
}

impl Error for ExecutionError {}

/// Per-payload failure reason returned inside receipts instead of aborting the
/// entire batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptFailure {
    InvalidPayload(&'static str),
    LaneUnavailable(String),
    LaneDisabled(String),
    PayloadTooLarge {
        lane_id: String,
        bytes: usize,
        max: usize,
    },
    GasLimitExceeded {
        requested: Gas,
        max: Gas,
    },
    IntrinsicGasTooHigh {
        intrinsic: Gas,
        provided_limit: Gas,
    },
    BlockGasExhausted {
        attempted: Gas,
        max_per_block: Gas,
    },
    InvalidSignature,
    ChainIdMismatch {
        expected: u64,
        got: u64,
    },
    ReplayDomainMismatch {
        expected: String,
        got: String,
    },
    TransactionExpired {
        timestamp: u64,
        expiration_timestamp: u64,
    },
    NonceGap {
        expected: u64,
        got: u64,
    },
    UnauthorizedLaneDispatch {
        sender: [u8; 32],
        lane_id: String,
    },
    SenderTxLimitExceeded {
        lane_id: String,
        sender: [u8; 32],
        max: usize,
    },
    TraceTooLarge {
        bytes: usize,
        max: usize,
    },
}

impl fmt::Display for ReceiptFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPayload(reason) => write!(f, "invalid payload: {reason}"),
            Self::LaneUnavailable(lane) => write!(f, "execution lane '{lane}' is unavailable"),
            Self::LaneDisabled(lane) => write!(f, "execution lane '{lane}' is disabled"),
            Self::PayloadTooLarge {
                lane_id,
                bytes,
                max,
            } => write!(
                f,
                "payload too large for lane '{lane_id}': {bytes} bytes > {max} bytes"
            ),
            Self::GasLimitExceeded { requested, max } => {
                write!(f, "payload gas limit exceeds policy: {requested} > {max}")
            }
            Self::IntrinsicGasTooHigh {
                intrinsic,
                provided_limit,
            } => write!(
                f,
                "intrinsic gas exceeds provided payload limit: {intrinsic} > {provided_limit}"
            ),
            Self::BlockGasExhausted {
                attempted,
                max_per_block,
            } => write!(
                f,
                "block gas exhausted: attempted cumulative {attempted} > block max {max_per_block}"
            ),
            Self::InvalidSignature => write!(f, "transaction signature failed validation"),
            Self::ChainIdMismatch { expected, got } => {
                write!(
                    f,
                    "transaction chain_id mismatch: expected {expected}, got {got}"
                )
            }
            Self::ReplayDomainMismatch { expected, got } => write!(
                f,
                "transaction replay domain mismatch: expected '{expected}', got '{got}'"
            ),
            Self::TransactionExpired {
                timestamp,
                expiration_timestamp,
            } => write!(
                f,
                "transaction expired at {expiration_timestamp}, current timestamp {timestamp}"
            ),
            Self::NonceGap { expected, got } => {
                write!(
                    f,
                    "sender nonce gap detected: expected {expected}, got {got}"
                )
            }
            Self::UnauthorizedLaneDispatch { sender, lane_id } => write!(
                f,
                "sender {} is not authorized for lane '{lane_id}'",
                hex::encode(sender)
            ),
            Self::SenderTxLimitExceeded {
                lane_id,
                sender,
                max,
            } => write!(
                f,
                "sender {} exceeded lane '{lane_id}' tx cap {max}",
                hex::encode(sender)
            ),
            Self::TraceTooLarge { bytes, max } => {
                write!(f, "execution trace too large: {bytes} bytes > {max} bytes")
            }
        }
    }
}

/// Deterministic execution context shared by the consensus layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub block_height: u64,
    pub timestamp: u64,
    pub max_gas_per_block: Gas,
    pub chain_id: u64,
    pub replay_domain: String,
    pub max_batch_tx_count: usize,
    pub max_batch_bytes: usize,
    pub max_receipt_size: usize,
    pub max_total_rejected_payloads_before_abort_threshold: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthScheme {
    Ed25519,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadType {
    Call,
    Deploy,
    System,
}

/// Payload forwarded from the consensus pipeline to a specific execution lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub version: u16,
    pub chain_id: u64,
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub gas_limit: Gas,
    pub max_fee: Gas,
    pub max_priority_fee: Gas,
    pub expiration_timestamp: u64,
    pub payload_type: PayloadType,
    pub access_scope: Vec<String>,
    pub replay_domain: String,
    pub auth_scheme: AuthScheme,
    pub signature: Vec<u8>,
    pub data: Vec<u8>,
}

impl ExecutionPayload {
    pub fn signing_digest(&self) -> Result<[u8; 32], ExecutionError> {
crate::engine::hash_payload_core(self)
    }

    pub fn sign_with_ed25519(mut self, signing_key: &SigningKey) -> Result<Self, ExecutionError> {
        self.sender = signing_key.verifying_key().to_bytes();
        let digest = self.signing_digest()?;
        self.signature = signing_key.sign(&digest).to_vec();
        Ok(self)
    }

    pub fn encoded_len(&self) -> Result<usize, ExecutionError> {
        crate::engine::canonical_bytes(self).map(|bytes| bytes.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOperation {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WriteSet {
    pub writes: Vec<WriteOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateDiffEntry {
    pub key: Vec<u8>,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StateDiff {
    pub entries: Vec<StateDiffEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostStateCommitment {
    pub state_root: [u8; 32],
    pub execution_trace_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub success: bool,
    pub gas_used: Gas,
    pub trace: Vec<String>,
    pub write_set: WriteSet,
    pub state_diff: StateDiff,
    pub commitment: PostStateCommitment,
}

/// Canonical execution receipt generated for every payload in the batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub success: bool,
    pub gas_used: Gas,
    pub cumulative_gas_used: Gas,
    pub state_root: [u8; 32],
    pub receipts_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub execution_trace_root: [u8; 32],
    pub error_message: Option<String>,
}

/// Batch-level deterministic accounting summary for audits and operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBatchSummary {
    pub receipt_count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub total_gas_used: Gas,
    pub block_gas_limit: Gas,
    pub rejected_count: usize,
    pub duplicate_tx_count: usize,
    pub nonce_violation_count: usize,
    pub lane_utilization: BTreeMap<String, Gas>,
    pub policy_versions: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchExecutionOutcome {
    pub receipts: Vec<ExecutionReceipt>,
    pub results: Vec<ExecutionResult>,
    pub summary: ExecutionBatchSummary,
    pub state_root: [u8; 32],
    pub receipt_root: [u8; 32],
    pub transactions_root: [u8; 32],
    pub execution_trace_root: [u8; 32],
    pub block_execution_root: [u8; 32],
}

pub trait StateStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn apply_write_set(&mut self, write_set: &WriteSet);
    fn snapshot_root(&self) -> Result<[u8; 32], ExecutionError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InMemoryStateStore {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl InMemoryStateStore {
    #[must_use]
    pub fn from_entries(entries: impl IntoIterator<Item = (Vec<u8>, Vec<u8>)>) -> Self {
        let data = entries.into_iter().collect();
        Self { data }
    }
}

impl StateStore for InMemoryStateStore {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    fn apply_write_set(&mut self, write_set: &WriteSet) {
        for write in &write_set.writes {
            self.data.insert(write.key.clone(), write.value.clone());
        }
    }

    fn snapshot_root(&self) -> Result<[u8; 32], ExecutionError> {
        let entries: Vec<(&Vec<u8>, &Vec<u8>)> = self.data.iter().collect();
crate::engine::merkle_like_root(DOMAIN_EXEC_STATE_V1, &entries)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneExecutionOutput {
    pub trace: Vec<String>,
    pub write_set: WriteSet,
}

pub trait ExecutionLane {
    fn lane_id(&self) -> &str;
    fn validate_payload(
        &self,
        context: &ExecutionContext,
        payload: &ExecutionPayload,
    ) -> Result<(), ReceiptFailure>;
    fn estimate_intrinsic_gas(
        &self,
        policy: &LanePolicy,
        payload: &ExecutionPayload,
    ) -> Result<Gas, ExecutionError>;
    fn execute(
        &self,
        context: &ExecutionContext,
        payload: &ExecutionPayload,
        pre_state: &dyn StateStore,
    ) -> Result<LaneExecutionOutput, ReceiptFailure>;
    fn verify_result(&self, output: &LaneExecutionOutput) -> Result<(), ReceiptFailure>;
    fn commit_changes(
        &self,
        output: &LaneExecutionOutput,
        state: &mut dyn StateStore,
    ) -> Result<StateDiff, ExecutionError>;
}

#[derive(Debug, Clone)]
