//! core/receipts/src/mod.rs
//!
//! AOXC Transaction Receipts.
//!
//! This module defines the canonical receipt structure produced by transaction
//! execution. Receipts provide deterministic evidence of execution outcomes
//! and are used to build the block receipt root.

use blake3::Hasher;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptHashError {
    LengthOverflow,
}

/// Canonical receipt hash size.
pub const HASH_SIZE: usize = 32;

/// Receipt hashing version.
pub const RECEIPT_HASH_VERSION: u8 = 1;

/// Global protocol namespace used for receipt-domain hash derivations.
const PROTOCOL_RECEIPT_NAMESPACE: &[u8] = b"AOXC/AOVM/RECEIPTS/HASH";

/// Receipt domain separator.
const DOMAIN_RECEIPT: &[u8] = b"RECEIPT";

/// Receipt root domain separator.
const DOMAIN_RECEIPT_ROOT: &[u8] = b"RECEIPT_ROOT";

/// Event domain separator.
const DOMAIN_EVENT: &[u8] = b"EVENT";

/// Canonical event structure.
///
/// Events allow execution layers to expose structured output that can be
/// indexed by external systems (explorers, analytics engines, bridges).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    /// Event type identifier.
    pub event_type: u16,

    /// Event payload.
    pub data: Vec<u8>,
}

/// Canonical transaction receipt.
///
/// A receipt is produced after execution of a transaction and records the
/// deterministic execution outcome.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    /// Canonical transaction identifier.
    pub transaction_hash: [u8; HASH_SIZE],

    /// Execution success flag.
    pub success: bool,

    /// Energy consumed during execution.
    pub energy_used: u64,

    /// Optional error code returned by the execution layer.
    pub error_code: Option<u16>,

    /// Structured execution events.
    pub events: Vec<Event>,
}

impl Receipt {
    /// Creates a successful receipt.
    #[must_use]
    pub fn success(tx_hash: [u8; HASH_SIZE], energy_used: u64) -> Self {
        Self {
            transaction_hash: tx_hash,
            success: true,
            energy_used,
            error_code: None,
            events: Vec::new(),
        }
    }

    /// Creates a failed receipt.
    #[must_use]
    pub fn failure(tx_hash: [u8; HASH_SIZE], energy_used: u64, error_code: u16) -> Self {
        Self {
            transaction_hash: tx_hash,
            success: false,
            energy_used,
            error_code: Some(error_code),
            events: Vec::new(),
        }
    }

    /// Appends an event to the receipt.
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Computes canonical receipt hash.
    pub fn try_hash(&self) -> Result<[u8; HASH_SIZE], ReceiptHashError> {
        let mut hasher = new_tagged_hasher(DOMAIN_RECEIPT);

        update_bytes32(&mut hasher, &self.transaction_hash);
        update_bool(&mut hasher, self.success);
        update_u64(&mut hasher, self.energy_used);

        match self.error_code {
            Some(code) => {
                update_bool(&mut hasher, true);
                update_u16(&mut hasher, code);
            }
            None => {
                update_bool(&mut hasher, false);
            }
        }

        let event_count = checked_len(self.events.len())?;
        update_u32(&mut hasher, event_count);

        for event in &self.events {
            let event_hash = try_hash_event(event)?;
            update_bytes32(&mut hasher, &event_hash);
        }

        Ok(finalize_hash(hasher))
    }

    #[must_use]
    pub fn hash(&self) -> [u8; HASH_SIZE] {
        self.try_hash()
            .expect("RECEIPT_HASH: receipt exceeded canonical encoding limits")
    }
}

/// Returns a fresh BLAKE3 hasher initialized with namespace, domain, and
/// version tags.
#[inline]
fn new_tagged_hasher(domain: &[u8]) -> Hasher {
    let mut hasher = Hasher::new();
    hasher.update(PROTOCOL_RECEIPT_NAMESPACE);
    hasher.update(&[0x00]);
    hasher.update(domain);
    hasher.update(&[0x00]);
    hasher.update(&[RECEIPT_HASH_VERSION]);
    hasher
}

/// Encodes a `bool` into the hash stream.
#[inline]
fn update_bool(hasher: &mut Hasher, value: bool) {
    hasher.update(&[u8::from(value)]);
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

/// Encodes a variable-length byte slice using a `u32` length prefix.
#[inline]
fn checked_len(value: usize) -> Result<u32, ReceiptHashError> {
    u32::try_from(value).map_err(|_| ReceiptHashError::LengthOverflow)
}

#[inline]
fn update_bytes(hasher: &mut Hasher, value: &[u8]) -> Result<(), ReceiptHashError> {
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

/// Hashes a single event.
fn try_hash_event(event: &Event) -> Result<[u8; HASH_SIZE], ReceiptHashError> {
    let mut hasher = new_tagged_hasher(DOMAIN_EVENT);

    update_u16(&mut hasher, event.event_type);
    update_bytes(&mut hasher, &event.data)?;

    Ok(finalize_hash(hasher))
}

/// Computes deterministic receipts root for a block.
pub fn try_calculate_receipts_root(
    receipts: &[Receipt],
) -> Result<[u8; HASH_SIZE], ReceiptHashError> {
    if receipts.is_empty() {
        return Ok(empty_receipts_root());
    }

    let mut hasher = new_tagged_hasher(DOMAIN_RECEIPT_ROOT);

    let count = checked_len(receipts.len())?;
    update_u32(&mut hasher, count);

    for receipt in receipts {
        let receipt_hash = receipt.try_hash()?;
        update_bytes32(&mut hasher, &receipt_hash);
    }

    Ok(finalize_hash(hasher))
}

#[must_use]
pub fn calculate_receipts_root(receipts: &[Receipt]) -> [u8; HASH_SIZE] {
    try_calculate_receipts_root(receipts)
        .expect("RECEIPT_HASH: receipts root exceeded canonical encoding limits")
}

/// Returns canonical empty receipts root.
#[must_use]
pub fn empty_receipts_root() -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_RECEIPT_ROOT);
    update_u32(&mut hasher, 0);
    finalize_hash(hasher)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_receipt() -> Receipt {
        let mut receipt = Receipt::success([1u8; HASH_SIZE], 100);

        receipt.push_event(Event {
            event_type: 1,
            data: vec![1, 2, 3],
        });

        receipt
    }

    #[test]
    fn receipt_hash_is_deterministic() {
        let receipt = sample_receipt();

        let a = receipt.hash().expect("receipt hash must calculate");
        let b = receipt.hash().expect("receipt hash must calculate");

        assert_eq!(a, b);
    }

    #[test]
    fn receipt_hash_changes_with_events() {
        let r1 = sample_receipt();
        let mut r2 = sample_receipt();

        r2.push_event(Event {
            event_type: 2,
            data: vec![9],
        });

        assert_ne!(
            r1.hash().expect("receipt hash must calculate"),
            r2.hash().expect("receipt hash must calculate")
        );
    }

    #[test]
    fn receipts_root_is_deterministic() {
        let r1 = sample_receipt();
        let r2 = sample_receipt();

        let root_a =
            calculate_receipts_root(&[r1.clone(), r2.clone()]).expect("root must calculate");
        let root_b = calculate_receipts_root(&[r1, r2]).expect("root must calculate");

        assert_eq!(root_a, root_b);
    }

    #[test]
    fn receipts_root_changes_when_order_changes() {
        let r1 = sample_receipt();

        let mut r2 = Receipt::success([2u8; HASH_SIZE], 100);
        r2.push_event(Event {
            event_type: 1,
            data: vec![1, 2, 3],
        });

        let root_a =
            calculate_receipts_root(&[r1.clone(), r2.clone()]).expect("root must calculate");
        let root_b = calculate_receipts_root(&[r2, r1]).expect("root must calculate");

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn checked_len_rejects_usize_values_above_u32() {
        assert_eq!(
            checked_len((u32::MAX as usize) + 1),
            Err(ReceiptHashError::LengthOverflow)
        );
    }
}
