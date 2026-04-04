use crate::*;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use std::collections::BTreeSet;

use super::hashing::{hash_payload_core, hash_struct};

pub(crate) fn validate_context(context: &ExecutionContext) -> Result<(), ExecutionError> {
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

pub(crate) fn validate_batch_limits(
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

pub(crate) fn validate_payload_shape(payload: &ExecutionPayload) -> Result<(), ReceiptFailure> {
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

pub(crate) fn validate_transaction_auth(
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
    }
    Ok(())
}

pub(crate) fn validate_no_duplicate_transactions(
    payloads: &[ExecutionPayload],
) -> Result<(), ExecutionError> {
    let mut seen = BTreeSet::new();
    for payload in payloads {
        if !seen.insert(payload.tx_hash) {
            return Err(ExecutionError::DuplicateTransaction(payload.tx_hash));
        }
    }
    Ok(())
}

pub(crate) fn validate_no_duplicate_sender_nonce(
    payloads: &[ExecutionPayload],
) -> Result<(), ExecutionError> {
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

pub(crate) fn canonicalize_payloads(payloads: &[ExecutionPayload]) -> Vec<ExecutionPayload> {
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

pub(crate) fn validate_registry_checksum(policy: &LaneRegistryPolicy) -> Result<(), ExecutionError> {
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

pub(crate) fn truncate_error_message(message: String) -> String {
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
