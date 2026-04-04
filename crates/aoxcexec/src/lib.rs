// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use ed25519_dalek::{Signer, SigningKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
        let checksum = engine::hash_struct(
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
            engine::validate_registry_checksum(entry)?;
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
engine::hash_payload_core(self)
    }

    pub fn sign_with_ed25519(mut self, signing_key: &SigningKey) -> Result<Self, ExecutionError> {
        self.sender = signing_key.verifying_key().to_bytes();
        let digest = self.signing_digest()?;
        self.signature = signing_key.sign(&digest).to_vec();
        Ok(self)
    }

    pub fn encoded_len(&self) -> Result<usize, ExecutionError> {
        engine::canonical_bytes(self).map(|bytes| bytes.len())
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
engine::merkle_like_root(DOMAIN_EXEC_STATE_V1, &entries)
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
        let nonce_key = engine::sender_nonce_key(payload.sender, payload.nonce);
        if pre_state.get(&nonce_key).is_some() {
            return Err(ReceiptFailure::NonceGap {
                expected: payload.nonce.saturating_add(1),
                got: payload.nonce,
            });
        }

        let state_key = engine::state_key(&payload.lane_id, &payload.sender, payload.nonce);
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


mod engine;

pub use engine::{default_lane_registry, default_lanes};

/// Compatibility alias retained for existing users that still instantiate the
/// old placeholder orchestrator.
pub type PlaceholderOrchestrator = DeterministicOrchestrator;

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

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

    fn signing_key_from_seed(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn sample_payload(
        tx_hash: [u8; 32],
        signer_seed: u8,
        nonce: u64,
        lane_id: &str,
        gas_limit: Gas,
        size: usize,
    ) -> ExecutionPayload {
        let signing_key = signing_key_from_seed(signer_seed);
        ExecutionPayload {
            version: 1,
            chain_id: 42,
            tx_hash,
            lane_id: lane_id.to_string(),
            sender: signing_key.verifying_key().to_bytes(),
            nonce,
            gas_limit,
            max_fee: gas_limit,
            max_priority_fee: gas_limit / 10,
            expiration_timestamp: 1_735_689_900,
            payload_type: PayloadType::Call,
            access_scope: vec![lane_id.to_string()],
            replay_domain: "aoxc-mainnet".to_string(),
            auth_scheme: AuthScheme::Ed25519,
            signature: vec![0u8; 64],
            data: vec![7u8; size],
        }
        .sign_with_ed25519(&signing_key)
        .expect("signature generation should succeed")
    }

    #[test]
    fn successful_batch_execution_produces_real_commitments_and_results() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], 11, 0, "native", 50_000, 32),
            sample_payload([2; 32], 22, 0, "evm", 75_000, 64),
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
            sample_payload([9; 32], 1, 0, "native", 50_000, 12),
            sample_payload([9; 32], 2, 0, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result, Err(ExecutionError::DuplicateTransaction([9; 32])));
    }

    #[test]
    fn duplicate_sender_nonce_rejects_before_execution() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let sender = signing_key_from_seed(7).verifying_key().to_bytes();
        let payloads = vec![
            sample_payload([1; 32], 7, 4, "native", 50_000, 12),
            sample_payload([2; 32], 7, 4, "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(
            result,
            Err(ExecutionError::DuplicateSenderNonce { sender, nonce: 4 })
        );
    }

    #[test]
    fn invalid_signature_returns_failed_receipt_without_state_mutation() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let mut payload = sample_payload([3; 32], 9, 0, "native", 50_000, 12);
        payload.signature = vec![99u8; 64];
        let valid = sample_payload([4; 32], 10, 0, "native", 50_000, 12);

        let outcome = orchestrator
            .execute_batch(&context, &[payload, valid])
            .expect("batch should return receipts");

        assert_eq!(
            outcome
                .receipts
                .iter()
                .filter(|receipt| !receipt.success)
                .count(),
            1
        );
        assert_eq!(
            outcome
                .receipts
                .iter()
                .filter(|receipt| receipt.success)
                .count(),
            1
        );
        assert_eq!(outcome.results.len(), 1);
    }

    #[test]
    fn nonce_gap_yields_canonical_rejection() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], 8, 0, "native", 50_000, 12),
            sample_payload([2; 32], 8, 2, "native", 50_000, 12),
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
            &[sample_payload([1; 32], 1, 0, "native", 50_000, 8)],
        );

        assert!(matches!(
            result,
            Err(ExecutionError::ConfigChecksumMismatch { .. })
        ));
    }

    #[test]
    fn serialization_freeze_for_payload_v1_is_stable() {
        let payload = sample_payload([1; 32], 2, 3, "wasm", 90_000, 4);
        let encoded = serde_json::to_string(&payload).expect("serialization should succeed");
        let digest = payload.signing_digest().expect("payload digest");
        let signing_key = signing_key_from_seed(2);
        let expected_signature = signing_key.sign(&digest).to_bytes();
        let sender_json = signing_key
            .verifying_key()
            .to_bytes()
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let signature_json = expected_signature
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let expected = format!(
            "{{\"version\":1,\"chain_id\":42,\"tx_hash\":[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],\"lane_id\":\"wasm\",\"sender\":[{sender_json}],\"nonce\":3,\"gas_limit\":90000,\"max_fee\":90000,\"max_priority_fee\":9000,\"expiration_timestamp\":1735689900,\"payload_type\":\"Call\",\"access_scope\":[\"wasm\"],\"replay_domain\":\"aoxc-mainnet\",\"auth_scheme\":\"Ed25519\",\"signature\":[{signature_json}],\"data\":[7,7,7,7]}}",
        );
        assert_eq!(encoded, expected);
    }

    #[test]
    fn deterministic_replay_holds_across_payload_sizes() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        for payload_size in [1usize, 2, 7, 16, 31, 63] {
            let payloads = vec![
                sample_payload([1; 32], 3, 0, "native", 60_000, payload_size),
                sample_payload([2; 32], 4, 0, "evm", 80_000, payload_size),
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
            let mut invalid = sample_payload([7; 32], 8, 0, "native", 50_000, size);
            invalid.signature = vec![0u8; 64];
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
        let payloads = vec![sample_payload([1; 32], 2, 0, "native", 50_000, 4)];
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
        let mut payload = sample_payload([5; 32], 6, 0, "native", 50_000, 8);
        payload.access_scope = vec!["native".to_string(), "native".to_string()];
        let signing_key = signing_key_from_seed(6);
        payload = payload
            .sign_with_ed25519(&signing_key)
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
        let payload = sample_payload([1; 32], 1, 0, "native", 50_000, 8);

        let err = orchestrator
            .execute_batch(&context, &[payload])
            .expect_err("invalid policy should fail");
        assert_eq!(
            err,
            ExecutionError::InvalidPolicy("base_gas must be greater than zero")
        );
    }
}
