use crate::*;
use blake3::Hasher;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use libcrux_ml_dsa::ml_dsa_65::{
    MLDSA65Signature, MLDSA65VerificationKey, verify as verify_ml_dsa_65,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const ML_DSA_65_SIGNATURE_SIZE: usize = 3309;
const ML_DSA_65_VERIFICATION_KEY_SIZE: usize = 1952;
const ML_DSA_65_CONTEXT: &[u8] = b"";
const DOMAIN_EXEC_PQ_ML_DSA_65_V1: &[u8] = b"AOXC_EXEC_PQ_ML_DSA_65_V1";

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
    match payload.auth_scheme {
        AuthScheme::Ed25519 => {
            if payload.pq_public_key.is_some() || payload.pq_signature.is_some() {
                return Err(ReceiptFailure::InvalidPayload(
                    "pq material is not allowed for Ed25519 transactions",
                ));
            }
        }
        AuthScheme::HybridEd25519MlDsa65 => {
            let Some(pq_public_key) = payload.pq_public_key.as_ref() else {
                return Err(ReceiptFailure::InvalidPayload(
                    "hybrid auth requires pq_public_key",
                ));
            };
            let Some(pq_signature) = payload.pq_signature.as_ref() else {
                return Err(ReceiptFailure::InvalidPayload(
                    "hybrid auth requires pq_signature",
                ));
            };
            if pq_public_key.len() != ML_DSA_65_VERIFICATION_KEY_SIZE {
                return Err(ReceiptFailure::InvalidPayload(
                    "hybrid pq_public_key has invalid length",
                ));
            }
            if pq_signature.len() != ML_DSA_65_SIGNATURE_SIZE {
                return Err(ReceiptFailure::InvalidPayload(
                    "hybrid pq_signature has invalid length",
                ));
            }
        }
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
    match payload.auth_scheme {
        AuthScheme::Ed25519 => {
            let verifying_key = VerifyingKey::from_bytes(&payload.sender)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
            let digest =
                hash_payload_core(payload).map_err(|_| ReceiptFailure::InvalidSignature)?;
            let signature_bytes: [u8; 64] = payload
                .signature
                .as_slice()
                .try_into()
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
            let signature = Signature::from_bytes(&signature_bytes);
            verifying_key
                .verify(&digest, &signature)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
        }
        AuthScheme::HybridEd25519MlDsa65 => {
            let verifying_key = VerifyingKey::from_bytes(&payload.sender)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
            let digest =
                hash_payload_core(payload).map_err(|_| ReceiptFailure::InvalidSignature)?;
            let signature_bytes: [u8; 64] = payload
                .signature
                .as_slice()
                .try_into()
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
            let signature = Signature::from_bytes(&signature_bytes);
            verifying_key
                .verify(&digest, &signature)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;

            let pq_public_key = payload
                .pq_public_key
                .as_ref()
                .ok_or(ReceiptFailure::InvalidSignature)?;
            let pq_signature = payload
                .pq_signature
                .as_ref()
                .ok_or(ReceiptFailure::InvalidSignature)?;

            let mut public_key_bytes = [0u8; ML_DSA_65_VERIFICATION_KEY_SIZE];
            public_key_bytes.copy_from_slice(pq_public_key);
            let public_key = MLDSA65VerificationKey::new(public_key_bytes);

            let mut signature_bytes = [0u8; ML_DSA_65_SIGNATURE_SIZE];
            signature_bytes.copy_from_slice(pq_signature);
            let signature = MLDSA65Signature::new(signature_bytes);
            let pq_message = hash_payload_ml_dsa_65_message(payload)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
            verify_ml_dsa_65(&public_key, &pq_message, ML_DSA_65_CONTEXT, &signature)
                .map_err(|_| ReceiptFailure::InvalidSignature)?;
        }
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

pub(crate) fn validate_registry_checksum(
    policy: &LaneRegistryPolicy,
) -> Result<(), ExecutionError> {
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

pub(crate) fn sender_nonce_key(sender: [u8; 32], nonce: u64) -> Vec<u8> {
    let mut key = b"nonce/".to_vec();
    key.extend_from_slice(&sender);
    key.extend_from_slice(&nonce.to_le_bytes());
    key
}

pub(crate) fn state_key(lane_id: &str, sender: &[u8; 32], nonce: u64) -> Vec<u8> {
    let mut key = b"state/".to_vec();
    key.extend_from_slice(lane_id.as_bytes());
    key.push(b'/');
    key.extend_from_slice(sender);
    key.extend_from_slice(&nonce.to_le_bytes());
    key
}

pub(crate) fn canonical_bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, ExecutionError> {
    serde_json::to_vec(value)
        .map_err(|_| ExecutionError::SerializationFailure("serde_json::to_vec failed"))
}

pub(crate) fn hash_struct<T: Serialize>(domain: &[u8], value: &T) -> [u8; 32] {
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

pub(crate) fn hash_payload_core(payload: &ExecutionPayload) -> Result<[u8; 32], ExecutionError> {
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

pub(crate) fn hash_payload_ml_dsa_65_message(
    payload: &ExecutionPayload,
) -> Result<Vec<u8>, ExecutionError> {
    hash_payload_core(payload).map(|core| {
        let mut message = Vec::with_capacity(DOMAIN_EXEC_PQ_ML_DSA_65_V1.len() + 1 + core.len());
        message.extend_from_slice(DOMAIN_EXEC_PQ_ML_DSA_65_V1);
        message.push(0x00);
        message.extend_from_slice(&core);
        message
    })
}

pub(crate) fn merkle_like_root<T: Serialize + ?Sized>(
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
