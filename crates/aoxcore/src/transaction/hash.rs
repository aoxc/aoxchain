//! core/transaction/src/hash.rs
//!
//! AOXC Transaction Cryptographic Sealing Module.
//!
//! This module provides deterministic, domain-separated, versioned hashing
//! primitives for transaction intents, signed transactions, signing payloads,
//! and transaction collections.
//!
//! Design objectives:
//! - Stable hash outputs across platforms
//! - Explicit domain separation across transaction hash classes
//! - Forward-compatible encoding discipline
//! - Clean separation between unsigned intent identity and signed transaction identity
//! - Clear extension path for future collection commitments

use blake3::Hasher;

use super::Transaction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionHashError {
    LengthOverflow,
}

impl From<std::num::TryFromIntError> for TransactionHashError {
    fn from(_: std::num::TryFromIntError) -> Self {
        Self::LengthOverflow
    }
}

/// Canonical hash output size in bytes.
pub const HASH_SIZE: usize = 32;

/// Canonical all-zero hash constant.
///
/// This value is exposed as a sentinel constant, but structured empty roots
/// should prefer [`empty_transaction_root`].
pub const ZERO_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

/// Version tag for the transaction hashing format.
///
/// This value must be incremented if any canonical transaction hashing layout
/// changes in a backward-incompatible manner.
pub const HASH_FORMAT_VERSION: u8 = 1;

/// Global protocol namespace used for all transaction-domain hash derivations.
const PROTOCOL_HASH_NAMESPACE: &[u8] = b"AOXC/AOVM/TRANSACTION/HASH";

/// Domain separator for generic transaction-domain hashing.
const DOMAIN_GENERIC: &[u8] = b"GENERIC";

/// Domain separator for canonical signing-payload hashing.
const DOMAIN_SIGNING_PAYLOAD: &[u8] = b"SIGNING_PAYLOAD";

/// Domain separator for unsigned transaction intent hashing.
const DOMAIN_TX_INTENT: &[u8] = b"INTENT";

/// Domain separator for signed transaction hashing.
const DOMAIN_TX_SIGNED: &[u8] = b"SIGNED";

/// Domain separator for transaction collection hashing.
const DOMAIN_TX_ROOT: &[u8] = b"ROOT";

/// Domain separator for the canonical empty transaction root.
const DOMAIN_EMPTY_TX_ROOT: &[u8] = b"EMPTY_ROOT";

/// Domain separator for transaction leaf hashing.
const DOMAIN_TX_LEAF: &[u8] = b"LEAF";

/// Domain separator for future internal-node hashing.
const DOMAIN_TX_INTERNAL: &[u8] = b"INTERNAL";

/// Returns a fresh BLAKE3 hasher initialized with namespace, domain, and
/// version tags.
#[inline]
fn new_tagged_hasher(domain: &[u8]) -> Hasher {
    let mut hasher = Hasher::new();
    hasher.update(PROTOCOL_HASH_NAMESPACE);
    hasher.update(&[0x00]);
    hasher.update(domain);
    hasher.update(&[0x00]);
    hasher.update(&[HASH_FORMAT_VERSION]);
    hasher
}

/// Encodes a `u8` into the hash stream.
#[inline]
fn update_u8(hasher: &mut Hasher, value: u8) {
    hasher.update(&[value]);
}

/// Encodes a `u16` into the hash stream using little-endian order.
#[inline]
fn update_u16(hasher: &mut Hasher, value: u16) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a `u32` into the hash stream using little-endian order.
#[inline]
fn update_u32(hasher: &mut Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a `u64` into the hash stream using little-endian order.
#[inline]
fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a fixed 32-byte value into the hash stream.
#[inline]
fn update_bytes32(hasher: &mut Hasher, value: &[u8; HASH_SIZE]) {
    hasher.update(value);
}

/// Encodes a fixed 64-byte value into the hash stream.
#[inline]
fn update_bytes64(hasher: &mut Hasher, value: &[u8; 64]) {
    hasher.update(value);
}

/// Encodes a variable-length byte slice using a `u32` length prefix.
#[inline]
fn checked_len(value: usize) -> Result<u32, TransactionHashError> {
    u32::try_from(value).map_err(|_| TransactionHashError::LengthOverflow)
}

#[inline]
fn update_bytes(hasher: &mut Hasher, value: &[u8]) -> Result<(), TransactionHashError> {
    let len = checked_len(value.len())?;
    update_u32(hasher, len);
    hasher.update(value);
    Ok(())
}

/// Finalizes the hasher into the canonical 32-byte digest format.
#[inline]
fn finalize_hash(hasher: Hasher) -> [u8; HASH_SIZE] {
    *hasher.finalize().as_bytes()
}

/// Computes a generic domain-separated BLAKE3 hash over an arbitrary byte slice.
///
/// This helper is suitable for transaction-adjacent standalone byte payloads,
/// but structured transaction objects should use dedicated functions.
pub fn try_compute_hash(data: &[u8]) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    let mut hasher = new_tagged_hasher(DOMAIN_GENERIC);
    update_bytes(&mut hasher, data)?;
    Ok(finalize_hash(hasher))
}

pub fn compute_hash(data: &[u8]) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_compute_hash(data)
}

/// Returns the canonical empty transaction root.
///
/// This is preferred over a raw zero hash because it remains inside the
/// structured transaction hashing namespace.
#[must_use]
pub fn empty_transaction_root() -> [u8; HASH_SIZE] {
    let hasher = new_tagged_hasher(DOMAIN_EMPTY_TX_ROOT);
    finalize_hash(hasher)
}

/// Hashes the canonical signing payload semantic content of a transaction.
///
/// This is distinct from the raw `Transaction::signing_message()` byte vector
/// and intentionally bound to a dedicated hashing namespace.
pub fn try_hash_signing_payload(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    let mut hasher = new_tagged_hasher(DOMAIN_SIGNING_PAYLOAD);

    update_bytes32(&mut hasher, &tx.sender);
    update_u64(&mut hasher, tx.nonce);
    update_u8(&mut hasher, tx.capability.code());
    update_u16(&mut hasher, tx.target.code());
    update_bytes(&mut hasher, &tx.payload)?;

    Ok(finalize_hash(hasher))
}

pub fn hash_signing_payload(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_hash_signing_payload(tx)
}

/// Computes the canonical unsigned transaction intent hash.
///
/// This hash excludes the signature and is suitable for:
/// - pre-signing caching
/// - semantic deduplication of unsigned commands
/// - intent-level indexing
pub fn try_hash_transaction_intent(
    tx: &Transaction,
) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    let mut hasher = new_tagged_hasher(DOMAIN_TX_INTENT);

    update_bytes32(&mut hasher, &tx.sender);
    update_u64(&mut hasher, tx.nonce);
    update_u8(&mut hasher, tx.capability.code());
    update_u16(&mut hasher, tx.target.code());
    update_bytes(&mut hasher, &tx.payload)?;

    Ok(finalize_hash(hasher))
}

pub fn hash_transaction_intent(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_hash_transaction_intent(tx)
}

/// Computes the canonical signed transaction hash.
///
/// This hash includes the signature and serves as the stable identifier of the
/// fully sealed transaction object.
pub fn try_hash_transaction(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    let mut hasher = new_tagged_hasher(DOMAIN_TX_SIGNED);

    update_bytes32(&mut hasher, &tx.sender);
    update_u64(&mut hasher, tx.nonce);
    update_u8(&mut hasher, tx.capability.code());
    update_u16(&mut hasher, tx.target.code());
    update_bytes(&mut hasher, &tx.payload)?;
    update_bytes64(&mut hasher, &tx.signature);

    Ok(finalize_hash(hasher))
}

pub fn hash_transaction(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_hash_transaction(tx)
}

/// Computes the canonical transaction leaf hash used in collection-root
/// aggregation.
///
/// The leaf namespace is intentionally distinct from the standalone signed
/// transaction hash namespace to preserve future flexibility.
pub fn try_hash_transaction_leaf(
    tx: &Transaction,
) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    let tx_hash = try_hash_transaction(tx)?;

    let mut hasher = new_tagged_hasher(DOMAIN_TX_LEAF);
    update_bytes32(&mut hasher, &tx_hash);

    Ok(finalize_hash(hasher))
}

pub fn hash_transaction_leaf(tx: &Transaction) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_hash_transaction_leaf(tx)
}

/// Computes the canonical commitment root for a slice of transactions.
///
/// Current strategy:
/// - Empty slice => structured empty root
/// - Non-empty slice => deterministic linear aggregation of transaction leaves
///
/// This aggregation model is intentionally simple and stable. A future Merkle
/// construction can be introduced under a newer hash format version without
/// silently changing current outputs.
pub fn try_calculate_transaction_root(
    transactions: &[Transaction],
) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    if transactions.is_empty() {
        return Ok(empty_transaction_root());
    }

    let mut hasher = new_tagged_hasher(DOMAIN_TX_ROOT);

    let tx_count = checked_len(transactions.len())?;
    update_u32(&mut hasher, tx_count);

    for tx in transactions {
        let leaf = try_hash_transaction_leaf(tx)?;
        update_bytes32(&mut hasher, &leaf);
    }

    Ok(finalize_hash(hasher))
}

pub fn calculate_transaction_root(
    transactions: &[Transaction],
) -> Result<[u8; HASH_SIZE], TransactionHashError> {
    try_calculate_transaction_root(transactions)
}

/// Computes a future-reserved internal-node hash for transaction tree
/// constructions.
#[must_use]
pub fn hash_internal_node(left: &[u8; HASH_SIZE], right: &[u8; HASH_SIZE]) -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_TX_INTERNAL);
    update_bytes32(&mut hasher, left);
    update_bytes32(&mut hasher, right);
    finalize_hash(hasher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost};
    use crate::transaction::Transaction;

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    fn bytes64(v: u8) -> [u8; 64] {
        [v; 64]
    }

    fn sample_transaction(payload: Vec<u8>) -> Transaction {
        Transaction {
            sender: bytes32(1),
            nonce: 42,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload,
            signature: bytes64(9),
        }
    }

    #[test]
    fn signing_payload_hash_is_deterministic() {
        let tx = sample_transaction(vec![1, 2, 3, 4]);
        assert_eq!(hash_signing_payload(&tx), hash_signing_payload(&tx));
    }

    #[test]
    fn intent_hash_is_stable_and_signature_independent() {
        let a = sample_transaction(vec![7, 8, 9]);
        let mut b = sample_transaction(vec![7, 8, 9]);
        b.signature = bytes64(77);

        assert_eq!(hash_transaction_intent(&a), hash_transaction_intent(&b));
        assert_ne!(hash_transaction(&a), hash_transaction(&b));
    }

    #[test]
    fn signed_transaction_hash_changes_with_payload() {
        let a = sample_transaction(vec![1, 2, 3]);
        let b = sample_transaction(vec![1, 2, 3, 4]);

        assert_ne!(hash_transaction(&a), hash_transaction(&b));
    }

    #[test]
    fn empty_transaction_root_is_stable_and_non_zero() {
        let a = calculate_transaction_root(&[]).expect("root must calculate");
        let b = calculate_transaction_root(&[]).expect("root must calculate");

        assert_eq!(a, b);
        assert_eq!(a, empty_transaction_root());
        assert_ne!(a, ZERO_HASH);
    }

    #[test]
    fn transaction_root_changes_when_order_changes() {
        let tx1 = sample_transaction(vec![1]);
        let tx2 = sample_transaction(vec![2]);

        let root_a =
            calculate_transaction_root(&[tx1.clone(), tx2.clone()]).expect("root must calculate");
        let root_b = calculate_transaction_root(&[tx2, tx1]).expect("root must calculate");

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn generic_hash_is_domain_separated_from_signed_transaction_hash() {
        let payload = vec![1, 2, 3, 4];
        let tx = sample_transaction(payload.clone());

        assert_ne!(
            compute_hash(&payload).expect("hash must calculate"),
            hash_transaction(&tx).expect("hash must calculate")
        );
    }

    #[test]
    fn internal_node_hash_is_deterministic() {
        let left = bytes32(11);
        let right = bytes32(22);

        assert_eq!(
            hash_internal_node(&left, &right),
            hash_internal_node(&left, &right)
        );
    }

    #[test]
    fn checked_len_rejects_usize_values_above_u32() {
        assert_eq!(
            checked_len((u32::MAX as usize) + 1),
            Err(TransactionHashError::LengthOverflow)
        );
    }
}
