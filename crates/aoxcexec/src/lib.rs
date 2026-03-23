use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

/// Canonical gas unit for execution accounting.
pub type Gas = u64;

/// Deterministic execution-lane policy enforced by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePolicy {
    pub lane_id: String,
    pub enabled: bool,
    pub base_gas: Gas,
    pub gas_per_byte: Gas,
    pub max_payload_bytes: usize,
    pub max_gas_per_tx: Gas,
}

impl LanePolicy {
    #[must_use]
    pub fn new(
        lane_id: impl Into<String>,
        base_gas: Gas,
        gas_per_byte: Gas,
        max_payload_bytes: usize,
        max_gas_per_tx: Gas,
    ) -> Self {
        Self {
            lane_id: lane_id.into(),
            enabled: true,
            base_gas,
            gas_per_byte,
            max_payload_bytes,
            max_gas_per_tx,
        }
    }
}

/// Execution errors that should reject the batch before receipt generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionError {
    InvalidContext(&'static str),
    DuplicateTransaction([u8; 32]),
    ArithmeticOverflow,
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidContext(reason) => write!(f, "invalid execution context: {reason}"),
            Self::DuplicateTransaction(tx_hash) => {
                write!(
                    f,
                    "duplicate transaction in batch: {}",
                    hex::encode(tx_hash)
                )
            }
            Self::ArithmeticOverflow => write!(f, "arithmetic overflow in execution accounting"),
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
        }
    }
}

/// Deterministic execution context shared by the consensus layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ExecutionContext {
    pub block_height: u64,
    pub timestamp: u64,
    pub max_gas_per_block: Gas,
}

/// Payload forwarded from the consensus pipeline to a specific execution lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPayload {
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub gas_limit: Gas,
    pub data: Vec<u8>,
}

/// Canonical execution receipt generated for every payload in the batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub tx_hash: [u8; 32],
    pub lane_id: String,
    pub success: bool,
    pub gas_used: Gas,
    pub cumulative_gas_used: Gas,
    pub state_root_hint: [u8; 32],
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
}

pub trait ExecutionOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<Vec<ExecutionReceipt>, ExecutionError>;
}

/// Audit-oriented deterministic orchestrator that enforces lane policy before
/// handing execution to downstream runtimes.
#[derive(Debug, Clone)]
pub struct DeterministicOrchestrator {
    lane_policies: BTreeMap<String, LanePolicy>,
}

impl Default for DeterministicOrchestrator {
    fn default() -> Self {
        Self::new(default_lane_policies())
    }
}

impl DeterministicOrchestrator {
    #[must_use]
    pub fn new(policies: impl IntoIterator<Item = LanePolicy>) -> Self {
        let lane_policies = policies
            .into_iter()
            .map(|policy| (policy.lane_id.clone(), policy))
            .collect();
        Self { lane_policies }
    }

    #[must_use]
    pub fn lane_policy(&self, lane_id: &str) -> Option<&LanePolicy> {
        self.lane_policies.get(lane_id)
    }

    pub fn summarize_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<ExecutionBatchSummary, ExecutionError> {
        let receipts = self.execute_batch(context, payloads)?;
        Ok(summarize_receipts(context.max_gas_per_block, &receipts))
    }
}

impl ExecutionOrchestrator for DeterministicOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<Vec<ExecutionReceipt>, ExecutionError> {
        validate_context(context)?;
        validate_no_duplicate_transactions(payloads)?;

        let mut receipts = Vec::with_capacity(payloads.len());
        let mut cumulative_gas: Gas = 0;

        for payload in payloads {
            let receipt = match self.evaluate_payload(context, payload, cumulative_gas)? {
                PayloadEvaluation::Accepted {
                    gas_used,
                    cumulative_after,
                    state_root_hint,
                } => {
                    cumulative_gas = cumulative_after;
                    ExecutionReceipt {
                        tx_hash: payload.tx_hash,
                        lane_id: payload.lane_id.clone(),
                        success: true,
                        gas_used,
                        cumulative_gas_used: cumulative_gas,
                        state_root_hint,
                        error_message: None,
                    }
                }
                PayloadEvaluation::Rejected(reason) => ExecutionReceipt {
                    tx_hash: payload.tx_hash,
                    lane_id: payload.lane_id.clone(),
                    success: false,
                    gas_used: 0,
                    cumulative_gas_used: cumulative_gas,
                    state_root_hint: [0u8; 32],
                    error_message: Some(reason.to_string()),
                },
            };
            receipts.push(receipt);
        }

        Ok(receipts)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PayloadEvaluation {
    Accepted {
        gas_used: Gas,
        cumulative_after: Gas,
        state_root_hint: [u8; 32],
    },
    Rejected(ReceiptFailure),
}

impl DeterministicOrchestrator {
    fn evaluate_payload(
        &self,
        context: &ExecutionContext,
        payload: &ExecutionPayload,
        cumulative_gas: Gas,
    ) -> Result<PayloadEvaluation, ExecutionError> {
        if payload.lane_id.trim().is_empty() {
            return Ok(PayloadEvaluation::Rejected(ReceiptFailure::InvalidPayload(
                "lane_id must not be empty",
            )));
        }
        if payload.data.is_empty() {
            return Ok(PayloadEvaluation::Rejected(ReceiptFailure::InvalidPayload(
                "payload data must not be empty",
            )));
        }
        if payload.gas_limit == 0 {
            return Ok(PayloadEvaluation::Rejected(ReceiptFailure::InvalidPayload(
                "gas_limit must be greater than zero",
            )));
        }

        let Some(policy) = self.lane_policies.get(&payload.lane_id) else {
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

        let payload_bytes_gas = policy
            .gas_per_byte
            .checked_mul(payload.data.len() as Gas)
            .ok_or(ExecutionError::ArithmeticOverflow)?;
        let intrinsic_gas = policy
            .base_gas
            .checked_add(payload_bytes_gas)
            .ok_or(ExecutionError::ArithmeticOverflow)?;

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

        Ok(PayloadEvaluation::Accepted {
            gas_used: intrinsic_gas,
            cumulative_after,
            state_root_hint: derive_state_root_hint(context, payload),
        })
    }
}

#[must_use]
pub fn default_lane_policies() -> Vec<LanePolicy> {
    vec![
        LanePolicy::new("native", 21_000, 8, 64 * 1024, 5_000_000),
        LanePolicy::new("evm", 21_000, 16, 128 * 1024, 15_000_000),
        LanePolicy::new("wasm", 35_000, 24, 256 * 1024, 20_000_000),
        LanePolicy::new("sui_move", 40_000, 20, 128 * 1024, 12_000_000),
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

fn derive_state_root_hint(context: &ExecutionContext, payload: &ExecutionPayload) -> [u8; 32] {
    let mut state_root_hint = [0u8; 32];

    for (index, byte) in context.block_height.to_le_bytes().iter().enumerate() {
        state_root_hint[index] ^= *byte;
    }
    for (index, byte) in context.timestamp.to_le_bytes().iter().enumerate() {
        state_root_hint[8 + index] ^= *byte;
    }
    for (index, byte) in payload.tx_hash.iter().enumerate() {
        state_root_hint[index % 32] ^= *byte;
    }
    for (index, byte) in payload.lane_id.as_bytes().iter().enumerate() {
        state_root_hint[index % 32] ^= *byte;
    }
    for (index, byte) in payload.data.iter().enumerate() {
        state_root_hint[index % 32] ^= *byte;
    }

    state_root_hint
}

fn summarize_receipts(
    block_gas_limit: Gas,
    receipts: &[ExecutionReceipt],
) -> ExecutionBatchSummary {
    let success_count = receipts.iter().filter(|receipt| receipt.success).count();
    let total_gas_used = receipts
        .iter()
        .fold(0_u64, |acc, receipt| acc.saturating_add(receipt.gas_used));

    ExecutionBatchSummary {
        receipt_count: receipts.len(),
        success_count,
        failure_count: receipts.len().saturating_sub(success_count),
        total_gas_used,
        block_gas_limit,
    }
}

/// Compatibility alias retained for existing users that still instantiate the
/// old placeholder orchestrator.
pub type PlaceholderOrchestrator = DeterministicOrchestrator;

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_payload(
        tx_hash: [u8; 32],
        lane_id: &str,
        gas_limit: Gas,
        size: usize,
    ) -> ExecutionPayload {
        ExecutionPayload {
            tx_hash,
            lane_id: lane_id.to_string(),
            gas_limit,
            data: vec![7u8; size],
        }
    }

    fn sample_context() -> ExecutionContext {
        ExecutionContext {
            block_height: 7,
            timestamp: 1_735_689_600,
            max_gas_per_block: 200_000,
        }
    }

    #[test]
    fn successful_batch_execution_is_deterministic_and_accounted() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], "native", 50_000, 32),
            sample_payload([2; 32], "evm", 75_000, 64),
        ];

        let receipts = orchestrator
            .execute_batch(&context, &payloads)
            .expect("execution should succeed");

        assert_eq!(receipts.len(), 2);
        assert!(receipts.iter().all(|receipt| receipt.success));
        assert!(receipts[0].gas_used > 21_000);
        assert!(receipts[1].cumulative_gas_used > receipts[0].cumulative_gas_used);
        assert_ne!(receipts[0].state_root_hint, [0u8; 32]);
    }

    #[test]
    fn duplicate_transactions_reject_the_entire_batch() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([9; 32], "native", 50_000, 12),
            sample_payload([9; 32], "evm", 60_000, 16),
        ];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(result, Err(ExecutionError::DuplicateTransaction([9; 32])));
    }

    #[test]
    fn unknown_lane_returns_failed_receipt_without_aborting_batch() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], "native", 50_000, 12),
            sample_payload([2; 32], "zk-rollup", 50_000, 12),
        ];

        let receipts = orchestrator
            .execute_batch(&context, &payloads)
            .expect("batch should complete with failure receipt");

        assert!(receipts[0].success);
        assert!(!receipts[1].success);
        assert!(
            receipts[1]
                .error_message
                .as_deref()
                .is_some_and(|message| message.contains("unavailable"))
        );
    }

    #[test]
    fn block_gas_exhaustion_turns_late_payloads_into_failed_receipts() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = ExecutionContext {
            max_gas_per_block: 25_500,
            ..sample_context()
        };
        let payloads = vec![
            sample_payload([1; 32], "native", 25_000, 32),
            sample_payload([2; 32], "native", 25_000, 32),
        ];

        let receipts = orchestrator
            .execute_batch(&context, &payloads)
            .expect("batch should return receipts");

        assert!(receipts[0].success);
        assert!(!receipts[1].success);
        assert_eq!(receipts[1].gas_used, 0);
        assert_eq!(
            receipts[1].cumulative_gas_used,
            receipts[0].cumulative_gas_used
        );
    }

    #[test]
    fn summary_reports_success_failure_and_total_gas() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = sample_context();
        let payloads = vec![
            sample_payload([1; 32], "native", 50_000, 32),
            sample_payload([2; 32], "unknown", 50_000, 32),
        ];

        let summary = orchestrator
            .summarize_batch(&context, &payloads)
            .expect("summary should succeed");

        assert_eq!(summary.receipt_count, 2);
        assert_eq!(summary.success_count, 1);
        assert_eq!(summary.failure_count, 1);
        assert!(summary.total_gas_used > 0);
        assert_eq!(summary.block_gas_limit, context.max_gas_per_block);
    }

    #[test]
    fn invalid_context_is_rejected() {
        let orchestrator = DeterministicOrchestrator::default();
        let context = ExecutionContext::default();
        let payloads = vec![sample_payload([1; 32], "native", 50_000, 32)];

        let result = orchestrator.execute_batch(&context, &payloads);

        assert_eq!(
            result,
            Err(ExecutionError::InvalidContext(
                "block_height must be greater than zero"
            ))
        );
    }
}
