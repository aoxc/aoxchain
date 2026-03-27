use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Canonical gas unit for execution accounting.
pub type Gas = u64;

const DOMAIN_EXEC_PAYLOAD_V1: &[u8] = b"AOXC_EXEC_PAYLOAD_V1";
const DOMAIN_EXEC_RECEIPT_V1: &[u8] = b"AOXC_EXEC_RECEIPT_V1";
const DOMAIN_EXEC_STATE_V1: &[u8] = b"AOXC_EXEC_STATE_V1";
const DOMAIN_EXEC_TRACE_V1: &[u8] = b"AOXC_EXEC_TRACE_V1";
const DOMAIN_EXEC_BLOCK_V1: &[u8] = b"AOXC_EXEC_BLOCK_V1";
const DOMAIN_EXEC_CONFIG_V1: &[u8] = b"AOXC_EXEC_CONFIG_V1";
const MAX_ERROR_MESSAGE_LEN: usize = 160;

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
        let checksum = hash_struct(
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
            validate_registry_checksum(entry)?;
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
    MockBlake3,
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
    pub signature: [u8; 32],
    pub data: Vec<u8>,
}

impl ExecutionPayload {
    pub fn signing_digest(&self) -> Result<[u8; 32], ExecutionError> {
        hash_payload_core(self)
    }

    pub fn with_mock_signature(mut self) -> Result<Self, ExecutionError> {
        self.signature = self.signing_digest()?;
        Ok(self)
    }

    pub fn encoded_len(&self) -> Result<usize, ExecutionError> {
        canonical_bytes(self).map(|bytes| bytes.len())
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
        merkle_like_root(DOMAIN_EXEC_STATE_V1, &entries)
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
pub struct DeterministicLane {
    lane_id: String,
}

impl DeterministicLane {
    #[must_use]
    pub fn new(lane_id: impl Into<String>) -> Self {
        Self {
            lane_id: lane_id.into(),
        }
    }
}

impl ExecutionLane for DeterministicLane {
    fn lane_id(&self) -> &str {
        &self.lane_id
    }

    fn validate_payload(
        &self,
        _context: &ExecutionContext,
        payload: &ExecutionPayload,
    ) -> Result<(), ReceiptFailure> {
        if payload.lane_id != self.lane_id {
            return Err(ReceiptFailure::LaneUnavailable(payload.lane_id.clone()));
        }
        Ok(())
    }

    fn estimate_intrinsic_gas(
        &self,
        policy: &LanePolicy,
        payload: &ExecutionPayload,
    ) -> Result<Gas, ExecutionError> {
        let payload_bytes_gas = policy
            .gas_per_byte
            .checked_mul(payload.data.len() as Gas)
            .ok_or(ExecutionError::ArithmeticOverflow)?;
        policy
            .base_gas
            .checked_add(payload_bytes_gas)
            .ok_or(ExecutionError::ArithmeticOverflow)
    }

    fn execute(
        &self,
        _context: &ExecutionContext,
        payload: &ExecutionPayload,
        pre_state: &dyn StateStore,
    ) -> Result<LaneExecutionOutput, ReceiptFailure> {
        let nonce_key = sender_nonce_key(payload.sender, payload.nonce);
        if pre_state.get(&nonce_key).is_some() {
            return Err(ReceiptFailure::NonceGap {
                expected: payload.nonce.saturating_add(1),
                got: payload.nonce,
            });
        }

        let state_key = state_key(&payload.lane_id, &payload.sender, payload.nonce);
        let write_set = WriteSet {
            writes: vec![
                WriteOperation {
                    key: state_key,
                    value: payload.data.clone(),
                },
                WriteOperation {
                    key: nonce_key,
                    value: payload.nonce.to_le_bytes().to_vec(),
                },
            ],
        };
        let trace = vec![
            format!(
                "lane={} payload_type={:?}",
                payload.lane_id, payload.payload_type
            ),
            format!("sender={}", hex::encode(payload.sender)),
            format!("nonce={} bytes={}", payload.nonce, payload.data.len()),
        ];
        Ok(LaneExecutionOutput { trace, write_set })
    }

    fn verify_result(&self, output: &LaneExecutionOutput) -> Result<(), ReceiptFailure> {
        let trace_len: usize = output.trace.iter().map(String::len).sum();
        if trace_len > 2_048 {
            return Err(ReceiptFailure::TraceTooLarge {
                bytes: trace_len,
                max: 2_048,
            });
        }
        Ok(())
    }

    fn commit_changes(
        &self,
        output: &LaneExecutionOutput,
        state: &mut dyn StateStore,
    ) -> Result<StateDiff, ExecutionError> {
        let mut diff = Vec::with_capacity(output.write_set.writes.len());
        for write in &output.write_set.writes {
            diff.push(StateDiffEntry {
                key: write.key.clone(),
                before: state.get(&write.key),
                after: Some(write.value.clone()),
            });
        }
        state.apply_write_set(&output.write_set);
        Ok(StateDiff { entries: diff })
    }
}

pub trait ExecutionOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<BatchExecutionOutcome, ExecutionError>;
}

/// Audit-oriented deterministic orchestrator that enforces lane policy before
/// handing execution to downstream runtimes.
pub struct DeterministicOrchestrator {
    lane_registry: LaneRegistry,
    lanes: BTreeMap<String, Box<dyn ExecutionLane + Send + Sync>>,
    initial_state: InMemoryStateStore,
}

impl Default for DeterministicOrchestrator {
    fn default() -> Self {
        Self::new(
            default_lane_registry(),
            default_lanes(),
            InMemoryStateStore::default(),
        )
    }
}

impl DeterministicOrchestrator {
    #[must_use]
    pub fn new(
        lane_registry: LaneRegistry,
        lanes: impl IntoIterator<Item = Box<dyn ExecutionLane + Send + Sync>>,
        initial_state: InMemoryStateStore,
    ) -> Self {
        let lanes = lanes
            .into_iter()
            .map(|lane| (lane.lane_id().to_string(), lane))
            .collect();
        Self {
            lane_registry,
            lanes,
            initial_state,
        }
    }

    pub fn summarize_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<ExecutionBatchSummary, ExecutionError> {
        self.execute_batch(context, payloads)
            .map(|result| result.summary)
    }
}

impl ExecutionOrchestrator for DeterministicOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<BatchExecutionOutcome, ExecutionError> {
        validate_context(context)?;
        validate_batch_limits(context, payloads)?;
        validate_no_duplicate_transactions(payloads)?;
        validate_no_duplicate_sender_nonce(payloads)?;

        let ordered_payloads = canonicalize_payloads(payloads);
        let transactions_root = merkle_root_for_transactions(&ordered_payloads)?;
        let mut state = self.initial_state.clone();
        let mut receipts = Vec::with_capacity(ordered_payloads.len());
        let mut results = Vec::new();
        let mut cumulative_gas: Gas = 0;
        let mut rejected_count = 0usize;
        let mut nonce_violation_count = 0usize;
        let mut lane_utilization: BTreeMap<String, Gas> = BTreeMap::new();
        let mut sender_lane_counts: BTreeMap<(String, [u8; 32]), usize> = BTreeMap::new();
        let mut sender_next_nonce: BTreeMap<[u8; 32], u64> = BTreeMap::new();

        for payload in ordered_payloads {
            let payload_result = self.evaluate_and_apply_payload(
                context,
                &payload,
                &mut state,
                cumulative_gas,
                &mut sender_next_nonce,
                &mut sender_lane_counts,
            )?;

            match payload_result {
                PayloadEvaluation::Accepted {
                    gas_used,
                    cumulative_after,
                    trace_root,
                    result,
                    state_root,
                } => {
                    cumulative_gas = cumulative_after;
                    *lane_utilization.entry(payload.lane_id.clone()).or_default() =
                        lane_utilization
                            .get(&payload.lane_id)
                            .copied()
                            .unwrap_or_default()
                            .checked_add(gas_used)
                            .ok_or(ExecutionError::ArithmeticOverflow)?;
                    results.push(*result);
                    receipts.push(ExecutionReceipt {
                        tx_hash: payload.tx_hash,
                        lane_id: payload.lane_id,
                        sender: payload.sender,
                        nonce: payload.nonce,
                        success: true,
                        gas_used,
                        cumulative_gas_used: cumulative_gas,
                        state_root,
                        receipts_root: [0u8; 32],
                        transactions_root,
                        execution_trace_root: trace_root,
                        error_message: None,
                    });
                }
                PayloadEvaluation::Rejected(reason) => {
                    rejected_count = rejected_count.saturating_add(1);
                    if matches!(reason, ReceiptFailure::NonceGap { .. }) {
                        nonce_violation_count = nonce_violation_count.saturating_add(1);
                    }
                    if rejected_count > context.max_total_rejected_payloads_before_abort_threshold {
                        break;
                    }
                    receipts.push(ExecutionReceipt {
                        tx_hash: payload.tx_hash,
                        lane_id: payload.lane_id,
                        sender: payload.sender,
                        nonce: payload.nonce,
                        success: false,
                        gas_used: 0,
                        cumulative_gas_used: cumulative_gas,
                        state_root: state.snapshot_root()?,
                        receipts_root: [0u8; 32],
                        transactions_root,
                        execution_trace_root: [0u8; 32],
                        error_message: Some(truncate_error_message(reason.to_string())),
                    });
                }
            }
        }

        let state_root = state.snapshot_root()?;
        let receipt_root = merkle_root_for_receipts(&receipts)?;
        let execution_trace_root = merkle_root_for_results(&results)?;
        for receipt in &mut receipts {
            receipt.receipts_root = receipt_root;
        }
        let block_execution_root = hash_struct(
            DOMAIN_EXEC_BLOCK_V1,
            &(
                state_root,
                receipt_root,
                transactions_root,
                execution_trace_root,
            ),
        );

        let summary = ExecutionBatchSummary {
            receipt_count: receipts.len(),
            success_count: receipts.iter().filter(|receipt| receipt.success).count(),
            failure_count: receipts.iter().filter(|receipt| !receipt.success).count(),
            total_gas_used: cumulative_gas,
            block_gas_limit: context.max_gas_per_block,
            rejected_count,
            duplicate_tx_count: 0,
            nonce_violation_count,
            lane_utilization,
            policy_versions: self.lane_registry.policy_versions(context.block_height),
        };

        Ok(BatchExecutionOutcome {
            receipts,
            results,
            summary,
            state_root,
            receipt_root,
            transactions_root,
            execution_trace_root,
            block_execution_root,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PayloadEvaluation {
    Accepted {
        gas_used: Gas,
        cumulative_after: Gas,
        trace_root: [u8; 32],
        result: Box<ExecutionResult>,
        state_root: [u8; 32],
    },
    Rejected(ReceiptFailure),
}

impl DeterministicOrchestrator {
    fn evaluate_and_apply_payload(
        &self,
        context: &ExecutionContext,
        payload: &ExecutionPayload,
        state: &mut InMemoryStateStore,
        cumulative_gas: Gas,
        sender_next_nonce: &mut BTreeMap<[u8; 32], u64>,
        sender_lane_counts: &mut BTreeMap<(String, [u8; 32]), usize>,
    ) -> Result<PayloadEvaluation, ExecutionError> {
        if let Err(reason) = validate_payload_shape(payload) {
            return Ok(PayloadEvaluation::Rejected(reason));
        }
        if let Err(reason) = validate_transaction_auth(context, payload) {
            return Ok(PayloadEvaluation::Rejected(reason));
        }

        let Some(policy) = self
            .lane_registry
            .resolve(&payload.lane_id, context.block_height)?
        else {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::LaneUnavailable(payload.lane_id.clone()),
            ));
        };
        if !policy.enabled {
            return Ok(PayloadEvaluation::Rejected(ReceiptFailure::LaneDisabled(
                payload.lane_id.clone(),
            )));
        }
        if payload.data.len() > policy.max_payload_bytes {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::PayloadTooLarge {
                    lane_id: payload.lane_id.clone(),
                    bytes: payload.data.len(),
                    max: policy.max_payload_bytes,
                },
            ));
        }
        if payload.gas_limit > policy.max_gas_per_tx {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::GasLimitExceeded {
                    requested: payload.gas_limit,
                    max: policy.max_gas_per_tx,
                },
            ));
        }
        if !payload
            .access_scope
            .iter()
            .any(|scope| scope == &payload.lane_id)
        {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::UnauthorizedLaneDispatch {
                    sender: payload.sender,
                    lane_id: payload.lane_id.clone(),
                },
            ));
        }

        let count_key = (payload.lane_id.clone(), payload.sender);
        let sender_count = sender_lane_counts.get(&count_key).copied().unwrap_or(0);
        if sender_count >= policy.max_sender_txs_per_block {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::SenderTxLimitExceeded {
                    lane_id: payload.lane_id.clone(),
                    sender: payload.sender,
                    max: policy.max_sender_txs_per_block,
                },
            ));
        }

        let expected_nonce = sender_next_nonce
            .get(&payload.sender)
            .copied()
            .unwrap_or(payload.nonce);
        if payload.nonce != expected_nonce {
            return Ok(PayloadEvaluation::Rejected(ReceiptFailure::NonceGap {
                expected: expected_nonce,
                got: payload.nonce,
            }));
        }

        let Some(lane) = self.lanes.get(&payload.lane_id) else {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::LaneUnavailable(payload.lane_id.clone()),
            ));
        };
        if let Err(reason) = lane.validate_payload(context, payload) {
            return Ok(PayloadEvaluation::Rejected(reason));
        }
        let intrinsic_gas = lane.estimate_intrinsic_gas(policy, payload)?;
        if intrinsic_gas > payload.gas_limit {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::IntrinsicGasTooHigh {
                    intrinsic: intrinsic_gas,
                    provided_limit: payload.gas_limit,
                },
            ));
        }
        let cumulative_after = cumulative_gas
            .checked_add(intrinsic_gas)
            .ok_or(ExecutionError::ArithmeticOverflow)?;
        if cumulative_after > context.max_gas_per_block {
            return Ok(PayloadEvaluation::Rejected(
                ReceiptFailure::BlockGasExhausted {
                    attempted: cumulative_after,
                    max_per_block: context.max_gas_per_block,
                },
            ));
        }

        let output = match lane.execute(context, payload, state) {
            Ok(output) => output,
            Err(reason) => return Ok(PayloadEvaluation::Rejected(reason)),
        };
        if let Err(reason) = lane.verify_result(&output) {
            return Ok(PayloadEvaluation::Rejected(reason));
        }
        let state_diff = lane.commit_changes(&output, state)?;
        let state_root = state.snapshot_root()?;
        let trace_root = hash_struct(DOMAIN_EXEC_TRACE_V1, &output.trace);
        let result = Box::new(ExecutionResult {
            tx_hash: payload.tx_hash,
            lane_id: payload.lane_id.clone(),
            success: true,
            gas_used: intrinsic_gas,
            trace: output.trace,
            write_set: output.write_set,
            state_diff,
            commitment: PostStateCommitment {
                state_root,
                execution_trace_root: trace_root,
            },
        });

        sender_next_nonce.insert(payload.sender, payload.nonce.saturating_add(1));
        sender_lane_counts.insert(count_key, sender_count.saturating_add(1));

        Ok(PayloadEvaluation::Accepted {
            gas_used: intrinsic_gas,
            cumulative_after,
            trace_root,
            result,
            state_root,
        })
    }
}

#[must_use]
pub fn default_lane_registry() -> LaneRegistry {
    LaneRegistry::new(vec![
        LaneRegistryPolicy::new(
            "native-mainnet",
            1,
            1,
            "gov://bootstrap/native/v1",
            LanePolicy::new("native", 21_000, 8, 64 * 1024, 5_000_000, 64),
        ),
        LaneRegistryPolicy::new(
            "evm-mainnet",
            1,
            1,
            "gov://bootstrap/evm/v1",
            LanePolicy::new("evm", 21_000, 16, 128 * 1024, 15_000_000, 64),
        ),
        LaneRegistryPolicy::new(
            "wasm-mainnet",
            1,
            1,
            "gov://bootstrap/wasm/v1",
            LanePolicy::new("wasm", 35_000, 24, 256 * 1024, 20_000_000, 32),
        ),
        LaneRegistryPolicy::new(
            "sui-move-mainnet",
            1,
            1,
            "gov://bootstrap/sui_move/v1",
            LanePolicy::new("sui_move", 40_000, 20, 128 * 1024, 12_000_000, 32),
        ),
    ])
}

#[must_use]
pub fn default_lanes() -> Vec<Box<dyn ExecutionLane + Send + Sync>> {
    vec![
        Box::new(DeterministicLane::new("native")),
        Box::new(DeterministicLane::new("evm")),
        Box::new(DeterministicLane::new("wasm")),
        Box::new(DeterministicLane::new("sui_move")),
    ]
}

fn validate_context(context: &ExecutionContext) -> Result<(), ExecutionError> {
    if context.block_height == 0 {
        return Err(ExecutionError::InvalidContext(
            "block_height must be greater than zero",
        ));
    }
    if context.timestamp == 0 {
        return Err(ExecutionError::InvalidContext(
            "timestamp must be greater than zero",
        ));
    }
    if context.max_gas_per_block == 0 {
        return Err(ExecutionError::InvalidContext(
            "max_gas_per_block must be greater than zero",
        ));
    }
    if context.chain_id == 0 {
        return Err(ExecutionError::InvalidContext(
            "chain_id must be greater than zero",
        ));
    }
    if context.replay_domain.trim().is_empty() {
        return Err(ExecutionError::InvalidContext(
            "replay_domain must not be empty",
        ));
    }
    if context.max_batch_tx_count == 0 || context.max_batch_bytes == 0 {
        return Err(ExecutionError::InvalidContext(
            "batch limits must be greater than zero",
        ));
    }
    if context.max_receipt_size == 0 {
        return Err(ExecutionError::InvalidContext(
            "max_receipt_size must be greater than zero",
        ));
    }
    Ok(())
}

fn validate_batch_limits(
    context: &ExecutionContext,
    payloads: &[ExecutionPayload],
) -> Result<(), ExecutionError> {
    if payloads.len() > context.max_batch_tx_count {
        return Err(ExecutionError::InvalidContext(
            "payload count exceeds max_batch_tx_count",
        ));
    }
    let total_bytes = payloads.iter().try_fold(0usize, |acc, payload| {
        let payload_len = payload.encoded_len()?;
        acc.checked_add(payload_len)
            .ok_or(ExecutionError::ArithmeticOverflow)
    })?;
    if total_bytes > context.max_batch_bytes {
        return Err(ExecutionError::InvalidContext(
            "payload bytes exceed max_batch_bytes",
        ));
    }
    Ok(())
}

fn validate_payload_shape(payload: &ExecutionPayload) -> Result<(), ReceiptFailure> {
    if payload.version == 0 {
        return Err(ReceiptFailure::InvalidPayload(
            "payload version must be greater than zero",
        ));
    }
    if payload.lane_id.trim().is_empty() {
        return Err(ReceiptFailure::InvalidPayload("lane_id must not be empty"));
    }
    if !payload
        .lane_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
    {
        return Err(ReceiptFailure::InvalidPayload(
            "lane_id contains invalid characters",
        ));
    }
    if payload.sender == [0u8; 32] {
        return Err(ReceiptFailure::InvalidPayload("sender must not be zero"));
    }
    if payload.tx_hash == [0u8; 32] {
        return Err(ReceiptFailure::InvalidPayload("tx_hash must not be zero"));
    }
    if payload.data.is_empty() {
        return Err(ReceiptFailure::InvalidPayload(
            "payload data must not be empty",
        ));
    }
    if payload.gas_limit == 0 {
        return Err(ReceiptFailure::InvalidPayload(
            "gas_limit must be greater than zero",
        ));
    }
    if payload.max_fee < payload.max_priority_fee {
        return Err(ReceiptFailure::InvalidPayload(
            "max_fee must be >= max_priority_fee",
        ));
    }
    if payload.access_scope.is_empty() {
        return Err(ReceiptFailure::InvalidPayload(
            "access_scope must contain at least one lane",
        ));
    }
    let mut seen = BTreeSet::new();
    for scope in &payload.access_scope {
        let trimmed = scope.trim();
        if trimmed.is_empty() {
            return Err(ReceiptFailure::InvalidPayload(
                "access_scope items must not be empty",
            ));
        }
        if !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
        {
            return Err(ReceiptFailure::InvalidPayload(
                "access_scope items contain invalid characters",
            ));
        }
        if !seen.insert(trimmed) {
            return Err(ReceiptFailure::InvalidPayload(
                "access_scope items must be unique",
            ));
        }
    }
    if payload.replay_domain.trim().is_empty() {
        return Err(ReceiptFailure::InvalidPayload(
            "replay_domain must not be empty",
        ));
    }
    if payload.replay_domain.len() > 128 {
        return Err(ReceiptFailure::InvalidPayload("replay_domain is too long"));
    }
    Ok(())
}

fn validate_transaction_auth(
    context: &ExecutionContext,
    payload: &ExecutionPayload,
) -> Result<(), ReceiptFailure> {
    if payload.chain_id != context.chain_id {
        return Err(ReceiptFailure::ChainIdMismatch {
            expected: context.chain_id,
            got: payload.chain_id,
        });
    }
    if payload.replay_domain != context.replay_domain {
        return Err(ReceiptFailure::ReplayDomainMismatch {
            expected: context.replay_domain.clone(),
            got: payload.replay_domain.clone(),
        });
    }
    if payload.expiration_timestamp < context.timestamp {
        return Err(ReceiptFailure::TransactionExpired {
            timestamp: context.timestamp,
            expiration_timestamp: payload.expiration_timestamp,
        });
    }
    let expected_signature =
        hash_payload_core(payload).map_err(|_| ReceiptFailure::InvalidSignature)?;
    if payload.signature != expected_signature {
        return Err(ReceiptFailure::InvalidSignature);
    }
    Ok(())
}

fn validate_no_duplicate_transactions(payloads: &[ExecutionPayload]) -> Result<(), ExecutionError> {
    let mut seen = BTreeSet::new();
    for payload in payloads {
        if !seen.insert(payload.tx_hash) {
            return Err(ExecutionError::DuplicateTransaction(payload.tx_hash));
        }
    }
    Ok(())
}

fn validate_no_duplicate_sender_nonce(payloads: &[ExecutionPayload]) -> Result<(), ExecutionError> {
    let mut seen = BTreeSet::new();
    for payload in payloads {
        if !seen.insert((payload.sender, payload.nonce)) {
            return Err(ExecutionError::DuplicateSenderNonce {
                sender: payload.sender,
                nonce: payload.nonce,
            });
        }
    }
    Ok(())
}

fn canonicalize_payloads(payloads: &[ExecutionPayload]) -> Vec<ExecutionPayload> {
    let mut ordered = payloads.to_vec();
    ordered.sort_by(|left, right| {
        left.lane_id
            .cmp(&right.lane_id)
            .then_with(|| left.sender.cmp(&right.sender))
            .then_with(|| left.nonce.cmp(&right.nonce))
            .then_with(|| left.tx_hash.cmp(&right.tx_hash))
    });
    ordered
}

fn validate_registry_checksum(policy: &LaneRegistryPolicy) -> Result<(), ExecutionError> {
    policy.policy.validate()?;
    let expected = hash_struct(
        DOMAIN_EXEC_CONFIG_V1,
        &(
            &policy.policy_id,
            policy.policy_version,
            policy.activation_height,
            &policy.governance_approval_ref,
            &policy.policy,
        ),
    );
    if policy.checksum != expected {
        return Err(ExecutionError::ConfigChecksumMismatch {
            lane_id: policy.policy.lane_id.clone(),
            policy_version: policy.policy_version,
        });
    }
    Ok(())
}

fn sender_nonce_key(sender: [u8; 32], nonce: u64) -> Vec<u8> {
    let mut key = b"nonce/".to_vec();
    key.extend_from_slice(&sender);
    key.extend_from_slice(&nonce.to_le_bytes());
    key
}

fn state_key(lane_id: &str, sender: &[u8; 32], nonce: u64) -> Vec<u8> {
    let mut key = b"state/".to_vec();
    key.extend_from_slice(lane_id.as_bytes());
    key.push(b'/');
    key.extend_from_slice(sender);
    key.extend_from_slice(&nonce.to_le_bytes());
    key
}

fn canonical_bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, ExecutionError> {
    serde_json::to_vec(value)
        .map_err(|_| ExecutionError::SerializationFailure("serde_json::to_vec failed"))
}

fn hash_struct<T: Serialize>(domain: &[u8], value: &T) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(domain);
    match serde_json::to_vec(value) {
        Ok(bytes) => {
            hasher.update(&bytes);
        }
        Err(_) => {
            hasher.update(b"SERDE_JSON_ENCODE_FAILURE");
        }
    };
    *hasher.finalize().as_bytes()
}

fn hash_payload_core(payload: &ExecutionPayload) -> Result<[u8; 32], ExecutionError> {
    canonical_bytes(&(
        payload.version,
        payload.chain_id,
        payload.tx_hash,
        &payload.lane_id,
        payload.sender,
        payload.nonce,
        payload.gas_limit,
        payload.max_fee,
        payload.max_priority_fee,
        payload.expiration_timestamp,
        &payload.payload_type,
        &payload.access_scope,
        &payload.replay_domain,
        &payload.auth_scheme,
        &payload.data,
    ))
    .map(|bytes| {
        let mut hasher = Hasher::new();
        hasher.update(DOMAIN_EXEC_PAYLOAD_V1);
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    })
}

fn merkle_like_root<T: Serialize + ?Sized>(
    domain: &[u8],
    value: &T,
) -> Result<[u8; 32], ExecutionError> {
    canonical_bytes(value).map(|bytes| {
        let mut hasher = Hasher::new();
        hasher.update(domain);
        hasher.update(&bytes);
        *hasher.finalize().as_bytes()
    })
}

fn merkle_root_for_transactions(payloads: &[ExecutionPayload]) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_PAYLOAD_V1, payloads)
}

fn merkle_root_for_receipts(receipts: &[ExecutionReceipt]) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_RECEIPT_V1, receipts)
}

fn merkle_root_for_results(results: &[ExecutionResult]) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_TRACE_V1, results)
}

fn truncate_error_message(message: String) -> String {
    if message.len() <= MAX_ERROR_MESSAGE_LEN {
        return message;
    }
    let mut end = 0usize;
    for (idx, ch) in message.char_indices() {
        let next = idx + ch.len_utf8();
        if next > MAX_ERROR_MESSAGE_LEN {
            break;
        }
        end = next;
    }
    message[..end].to_string()
}

/// Compatibility alias retained for existing users that still instantiate the
/// old placeholder orchestrator.
pub type PlaceholderOrchestrator = DeterministicOrchestrator;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> ExecutionContext {
        ExecutionContext {
            block_height: 7,
            timestamp: 1_735_689_600,
            max_gas_per_block: 200_000,
            chain_id: 42,
            replay_domain: "aoxc-mainnet".to_string(),
            max_batch_tx_count: 128,
            max_batch_bytes: 1024 * 1024,
            max_receipt_size: 4096,
            max_total_rejected_payloads_before_abort_threshold: 16,
        }
    }

    fn sample_payload(
        tx_hash: [u8; 32],
        sender: [u8; 32],
        nonce: u64,
        lane_id: &str,
        gas_limit: Gas,
        size: usize,
    ) -> ExecutionPayload {
        ExecutionPayload {
            version: 1,
            chain_id: 42,
            tx_hash,
            lane_id: lane_id.to_string(),
            sender,
            nonce,
            gas_limit,
            max_fee: gas_limit,
            max_priority_fee: gas_limit / 10,
            expiration_timestamp: 1_735_689_900,
            payload_type: PayloadType::Call,
            access_scope: vec![lane_id.to_string()],
            replay_domain: "aoxc-mainnet".to_string(),
            auth_scheme: AuthScheme::MockBlake3,
            signature: [0u8; 32],
            data: vec![7u8; size],
        }
        .with_mock_signature()
        .expect("signature generation should succeed")
    }

    #[test]
    fn successful_batch_execution_produces_real_commitments_and_results() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], [11; 32], 0, "native", 50_000, 32),
            sample_payload([2; 32], [22; 32], 0, "evm", 75_000, 64),
        ];

        let outcome = orchestrator
            .execute_batch(&context, &payloads)
            .expect("execution should succeed");

        assert_eq!(outcome.receipts.len(), 2);
        assert_eq!(outcome.results.len(), 2);
        assert!(outcome.receipts.iter().all(|receipt| receipt.success));
        assert_ne!(outcome.state_root, [0u8; 32]);
        assert_ne!(outcome.receipt_root, [0u8; 32]);
        assert_ne!(outcome.transactions_root, [0u8; 32]);
        assert_ne!(outcome.execution_trace_root, [0u8; 32]);
        assert_ne!(outcome.block_execution_root, [0u8; 32]);
    }

    #[test]
    fn duplicate_transactions_reject_the_entire_batch() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([9; 32], [1; 32], 0, "native", 50_000, 12),
            sample_payload([9; 32], [2; 32], 0, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result, Err(ExecutionError::DuplicateTransaction([9; 32])));
    }

    #[test]
    fn duplicate_sender_nonce_rejects_before_execution() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], [7; 32], 4, "native", 50_000, 12),
            sample_payload([2; 32], [7; 32], 4, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(
            result,
            Err(ExecutionError::DuplicateSenderNonce {
                sender: [7; 32],
                nonce: 4,
            })
        );
    }

    #[test]
    fn invalid_signature_returns_failed_receipt_without_state_mutation() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let mut payload = sample_payload([3; 32], [9; 32], 0, "native", 50_000, 12);
        payload.signature = [99u8; 32];
        let valid = sample_payload([4; 32], [10; 32], 0, "native", 50_000, 12);

        let outcome = orchestrator
            .execute_batch(&context, &[payload, valid])
            .expect("batch should return receipts");

        assert!(!outcome.receipts[0].success);
        assert!(outcome.receipts[1].success);
        assert_eq!(outcome.results.len(), 1);
    }

    #[test]
    fn nonce_gap_yields_canonical_rejection() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], [8; 32], 0, "native", 50_000, 12),
            sample_payload([2; 32], [8; 32], 2, "native", 50_000, 12),
        ];

        let outcome = orchestrator
            .execute_batch(&context, &payloads)
            .expect("batch should complete");

        assert!(outcome.receipts[0].success);
        assert!(!outcome.receipts[1].success);
        assert_eq!(outcome.summary.nonce_violation_count, 1);
    }

    #[test]
    fn registry_checksum_mismatch_halts_execution() {
        let mut registry = default_lane_registry();
        let native_entries = registry
            .policies
            .get_mut("native")
            .expect("native policy exists");
        native_entries[0].checksum = [0u8; 32];
        let orchestrator = DeterministicOrchestrator::new(
            registry,
            default_lanes(),
            InMemoryStateStore::default(),
        );

        let result = orchestrator.execute_batch(
            &sample_context(),
            &[sample_payload([1; 32], [1; 32], 0, "native", 50_000, 8)],
        );

        assert!(matches!(
            result,
            Err(ExecutionError::ConfigChecksumMismatch { .. })
        ));
    }

    #[test]
    fn serialization_freeze_for_payload_v1_is_stable() {
        let payload = sample_payload([1; 32], [2; 32], 3, "wasm", 90_000, 4);
        let encoded = serde_json::to_string(&payload).expect("serialization should succeed");
        let expected = r#"{"version":1,"chain_id":42,"tx_hash":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],"lane_id":"wasm","sender":[2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],"nonce":3,"gas_limit":90000,"max_fee":90000,"max_priority_fee":9000,"expiration_timestamp":1735689900,"payload_type":"Call","access_scope":["wasm"],"replay_domain":"aoxc-mainnet","auth_scheme":"MockBlake3","signature":[193,13,89,183,6,112,76,169,214,16,109,241,230,96,153,120,56,68,240,95,46,23,223,24,42,205,160,172,67,253,25,57],"data":[7,7,7,7]}"#;
        assert_eq!(encoded, expected);
    }

    #[test]
    fn deterministic_replay_holds_across_payload_sizes() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        for payload_size in [1usize, 2, 7, 16, 31, 63] {
            let payloads = vec![
                sample_payload([1; 32], [3; 32], 0, "native", 60_000, payload_size),
                sample_payload([2; 32], [4; 32], 0, "evm", 80_000, payload_size),
            ];

            let left = orchestrator
                .execute_batch(&context, &payloads)
                .expect("left outcome");
            let right = orchestrator
                .execute_batch(&context, &payloads)
                .expect("right outcome");

            assert_eq!(left.receipts, right.receipts);
            assert_eq!(left.state_root, right.state_root);
            assert_eq!(left.receipt_root, right.receipt_root);
            assert_eq!(left.block_execution_root, right.block_execution_root);
        }
    }

    #[test]
    fn invalid_tx_never_mutates_state_across_sizes() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        for size in [1usize, 4, 8, 16, 31] {
            let mut invalid = sample_payload([7; 32], [8; 32], 0, "native", 50_000, size);
            invalid.signature = [0u8; 32];
            let outcome = orchestrator
                .execute_batch(&context, &[invalid])
                .expect("outcome");
            assert_eq!(outcome.results.len(), 0);
            assert_eq!(outcome.receipts.len(), 1);
            assert!(!outcome.receipts[0].success);
            assert_eq!(
                outcome.state_root,
                InMemoryStateStore::default().snapshot_root().expect("root")
            );
        }
    }

    #[test]
    fn invalid_context_rejects_zero_max_receipt_size() {
        let mut context = sample_context();
        context.max_receipt_size = 0;
        let orchestrator = DeterministicOrchestrator::default();
        let payloads = vec![sample_payload([1; 32], [2; 32], 0, "native", 50_000, 4)];
        let err = orchestrator
            .execute_batch(&context, &payloads)
            .expect_err("context must be rejected");
        assert_eq!(
            err,
            ExecutionError::InvalidContext("max_receipt_size must be greater than zero")
        );
    }

    #[test]
    fn invalid_scope_item_is_rejected() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let mut payload = sample_payload([5; 32], [6; 32], 0, "native", 50_000, 8);
        payload.access_scope = vec!["native".to_string(), "native".to_string()];
        payload = payload
            .with_mock_signature()
            .expect("signature generation should succeed");

        let outcome = orchestrator
            .execute_batch(&context, &[payload])
            .expect("batch should return receipt");
        assert_eq!(outcome.receipts.len(), 1);
        assert!(!outcome.receipts[0].success);
        assert!(
            outcome.receipts[0]
                .error_message
                .as_deref()
                .unwrap_or_default()
                .contains("access_scope")
        );
    }

    #[test]
    fn invalid_policy_configuration_is_rejected() {
        let bad_policy = LanePolicy {
            lane_id: "native".to_string(),
            enabled: true,
            base_gas: 0,
            gas_per_byte: 1,
            max_payload_bytes: 1024,
            max_gas_per_tx: 1_000_000,
            max_sender_txs_per_block: 10,
        };
        let registry = LaneRegistry::new(vec![LaneRegistryPolicy::new(
            "native-mainnet",
            1,
            1,
            "gov://bootstrap/native/v1",
            bad_policy,
        )]);
        let orchestrator = DeterministicOrchestrator::new(
            registry,
            default_lanes(),
            InMemoryStateStore::default(),
        );
        let context = sample_context();
        let payload = sample_payload([1; 32], [1; 32], 0, "native", 50_000, 8);

        let err = orchestrator
            .execute_batch(&context, &[payload])
            .expect_err("invalid policy should fail");
        assert_eq!(
            err,
            ExecutionError::InvalidPolicy("base_gas must be greater than zero")
        );
    }
}
