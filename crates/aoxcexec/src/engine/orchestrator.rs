use crate::*;
use std::collections::BTreeMap;

use super::hashing::{hash_struct, merkle_root_for_receipts, merkle_root_for_results, merkle_root_for_transactions};
use super::validation::{canonicalize_payloads, truncate_error_message, validate_batch_limits, validate_context, validate_no_duplicate_sender_nonce, validate_no_duplicate_transactions, validate_payload_shape, validate_transaction_auth};

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

