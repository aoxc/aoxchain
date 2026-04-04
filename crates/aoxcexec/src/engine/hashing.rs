use crate::*;
use blake3::Hasher;
use serde::Serialize;

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

pub(crate) fn merkle_root_for_transactions(
    payloads: &[ExecutionPayload],
) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_PAYLOAD_V1, payloads)
}

pub(crate) fn merkle_root_for_receipts(
    receipts: &[ExecutionReceipt],
) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_RECEIPT_V1, receipts)
}

pub(crate) fn merkle_root_for_results(
    results: &[ExecutionResult],
) -> Result<[u8; 32], ExecutionError> {
    merkle_like_root(DOMAIN_EXEC_TRACE_V1, results)
}
