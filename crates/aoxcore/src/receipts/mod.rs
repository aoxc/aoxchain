// AOXC MIT License
// Production-oriented receipt primitive for deterministic post-execution accounting.
// This module defines the canonical receipt model used to represent execution
// outcomes and to derive the block-level receipts root.
//
// Security objectives:
// - deterministic hashing
// - strict state validation
// - bounded variable-length fields
// - explicit domain separation
// - fail-closed construction discipline
//
// This module intentionally avoids embedding execution-engine-specific policy.
// It provides a canonical and auditable receipt representation that downstream
// execution, proof, settlement, and indexing layers may rely upon safely.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical receipt hash size in bytes.
pub const HASH_SIZE: usize = 32;

/// Receipt hashing version.
///
/// Any incompatible canonical encoding change must increment this value.
pub const RECEIPT_HASH_VERSION: u8 = 1;

/// Global protocol namespace used for receipt-domain hash derivations.
const PROTOCOL_RECEIPT_NAMESPACE: &[u8] = b"AOXC/AOVM/RECEIPTS/HASH";

/// Domain separator for a single event hash.
const DOMAIN_EVENT: &[u8] = b"EVENT";

/// Domain separator for a single receipt hash.
const DOMAIN_RECEIPT: &[u8] = b"RECEIPT";

/// Domain separator for block-level receipts root derivation.
const DOMAIN_RECEIPT_ROOT: &[u8] = b"RECEIPT_ROOT";

const ZERO_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

/// Upper bound for event payload bytes.
///
/// The limit is intentionally conservative to reduce abuse risk, bound hashing
/// work, and keep receipt artifacts operationally manageable for nodes,
/// explorers, indexers, and downstream integrations.
pub const MAX_EVENT_DATA_LEN: usize = 64 * 1024;

/// Upper bound for the number of events carried by a single receipt.
///
/// This guards against event amplification and stabilizes operational budgets
/// for hashing, indexing, persistence, and proof generation.
pub const MAX_EVENTS_PER_RECEIPT: usize = 1024;

/// Errors that may arise during canonical receipt validation or hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptError {
    LengthOverflow,
    ZeroTransactionHash,
    SuccessReceiptMustNotContainErrorCode,
    FailureReceiptMustContainErrorCode,
    EventDataTooLarge,
    TooManyEvents,
}

impl fmt::Display for ReceiptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => {
                f.write_str("canonical receipt encoding exceeds supported length bounds")
            }
            Self::ZeroTransactionHash => {
                f.write_str("transaction_hash must not be zero")
            }
            Self::SuccessReceiptMustNotContainErrorCode => {
                f.write_str("successful receipt must not contain an error_code")
            }
            Self::FailureReceiptMustContainErrorCode => {
                f.write_str("failed receipt must contain an error_code")
            }
            Self::EventDataTooLarge => {
                f.write_str("event data exceeds the maximum canonical size")
            }
            Self::TooManyEvents => {
                f.write_str("receipt contains more events than the canonical maximum")
            }
        }
    }
}

impl std::error::Error for ReceiptError {}

/// Canonical event structure.
///
/// Events expose structured execution output that may be indexed by external
/// systems such as explorers, analytics engines, bridging infrastructure,
/// accounting pipelines, and audit tooling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    /// Canonical event type identifier.
    pub event_type: u16,

    /// Opaque event payload bytes.
    pub data: Vec<u8>,
}

impl Event {
    /// Creates a new canonical event after validating bounded payload size.
    pub fn new(event_type: u16, data: Vec<u8>) -> Result<Self, ReceiptError> {
        if data.len() > MAX_EVENT_DATA_LEN {
            return Err(ReceiptError::EventDataTooLarge);
        }

        Ok(Self { event_type, data })
    }

    /// Validates a single event under canonical receipt rules.
    pub fn validate(&self) -> Result<(), ReceiptError> {
        if self.data.len() > MAX_EVENT_DATA_LEN {
            return Err(ReceiptError::EventDataTooLarge);
        }

        Ok(())
    }

    /// Computes the canonical event hash.
    pub fn try_hash(&self) -> Result<[u8; HASH_SIZE], ReceiptError> {
        self.validate()?;

        let mut hasher = new_tagged_hasher(DOMAIN_EVENT);
        update_u16(&mut hasher, self.event_type);
        update_bytes(&mut hasher, &self.data)?;

        Ok(finalize_hash(hasher))
    }

    /// Computes the canonical event hash and panics only if internal invariants
    /// were violated by unchecked construction or mutation.
    #[must_use]
    pub fn hash(&self) -> [u8; HASH_SIZE] {
        self.try_hash()
            .expect("receipt event hashing must operate on a previously validated canonical event")
    }
}

/// Canonical transaction receipt.
///
/// A receipt is produced after transaction execution and records the canonical
/// execution outcome. The model is intentionally compact and commits to the
/// minimal deterministic state required for:
/// - receipt hashing
/// - receipts-root derivation
/// - downstream verification
/// - explorer and indexer consumption
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Receipt {
    /// Canonical transaction identifier.
    pub transaction_hash: [u8; HASH_SIZE],

    /// Execution success flag.
    pub success: bool,

    /// Energy consumed during execution.
    pub energy_used: u64,

    /// Optional execution-layer error code.
    ///
    /// Canonical invariant:
    /// - `success == true`  => `error_code == None`
    /// - `success == false` => `error_code == Some(..)`
    pub error_code: Option<u16>,

    /// Structured execution events in canonical emission order.
    pub events: Vec<Event>,
}

impl Receipt {
    /// Creates a canonical successful receipt.
    pub fn success(tx_hash: [u8; HASH_SIZE], energy_used: u64) -> Result<Self, ReceiptError> {
        let receipt = Self {
            transaction_hash: tx_hash,
            success: true,
            energy_used,
            error_code: None,
            events: Vec::new(),
        };

        receipt.validate()?;
        Ok(receipt)
    }

    /// Creates a canonical failed receipt.
    pub fn failure(
        tx_hash: [u8; HASH_SIZE],
        energy_used: u64,
        error_code: u16,
    ) -> Result<Self, ReceiptError> {
        let receipt = Self {
            transaction_hash: tx_hash,
            success: false,
            energy_used,
            error_code: Some(error_code),
            events: Vec::new(),
        };

        receipt.validate()?;
        Ok(receipt)
    }

    /// Appends an event while preserving canonical receipt bounds.
    pub fn push_event(&mut self, event: Event) -> Result<(), ReceiptError> {
        event.validate()?;

        if self.events.len() >= MAX_EVENTS_PER_RECEIPT {
            return Err(ReceiptError::TooManyEvents);
        }

        self.events.push(event);
        Ok(())
    }

    /// Validates the receipt under canonical protocol rules.
    pub fn validate(&self) -> Result<(), ReceiptError> {
        if self.transaction_hash == ZERO_HASH {
            return Err(ReceiptError::ZeroTransactionHash);
        }

        match (self.success, self.error_code) {
            (true, Some(_)) => {
                return Err(ReceiptError::SuccessReceiptMustNotContainErrorCode);
            }
            (false, None) => {
                return Err(ReceiptError::FailureReceiptMustContainErrorCode);
            }
            _ => {}
        }

        if self.events.len() > MAX_EVENTS_PER_RECEIPT {
            return Err(ReceiptError::TooManyEvents);
        }

        for event in &self.events {
            event.validate()?;
        }

        Ok(())
    }

    /// Computes the canonical receipt hash.
    pub fn try_hash(&self) -> Result<[u8; HASH_SIZE], ReceiptError> {
        self.validate()?;

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
            let event_hash = event.try_hash()?;
            update_bytes32(&mut hasher, &event_hash);
        }

        Ok(finalize_hash(hasher))
    }

    /// Computes the canonical receipt hash and panics only if internal
    /// invariants were previously violated.
    #[must_use]
    pub fn hash(&self) -> [u8; HASH_SIZE] {
        self.try_hash()
            .expect("receipt hashing must operate on a previously validated canonical receipt")
    }

    /// Returns `true` when the receipt represents a failed execution outcome.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Returns `true` when the receipt contains no events.
    #[must_use]
    pub fn has_no_events(&self) -> bool {
        self.events.is_empty()
    }
}

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

#[inline]
fn update_bool(hasher: &mut Hasher, value: bool) {
    hasher.update(&[u8::from(value)]);
}

#[inline]
fn update_u16(hasher: &mut Hasher, value: u16) {
    hasher.update(&value.to_le_bytes());
}

#[inline]
fn update_u32(hasher: &mut Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

#[inline]
fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
}

#[inline]
fn update_bytes32(hasher: &mut Hasher, value: &[u8; HASH_SIZE]) {
    hasher.update(value);
}

#[inline]
fn checked_len(value: usize) -> Result<u32, ReceiptError> {
    u32::try_from(value).map_err(|_| ReceiptError::LengthOverflow)
}

#[inline]
fn update_bytes(hasher: &mut Hasher, value: &[u8]) -> Result<(), ReceiptError> {
    let len = checked_len(value.len())?;
    update_u32(hasher, len);
    hasher.update(value);
    Ok(())
}

#[inline]
fn finalize_hash(hasher: Hasher) -> [u8; HASH_SIZE] {
    *hasher.finalize().as_bytes()
}

/// Computes the deterministic receipts root for a block.
pub fn try_calculate_receipts_root(receipts: &[Receipt]) -> Result<[u8; HASH_SIZE], ReceiptError> {
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

/// Computes the deterministic receipts root and panics only if the caller
/// provided a previously invalid canonical receipt set.
#[must_use]
pub fn calculate_receipts_root(receipts: &[Receipt]) -> [u8; HASH_SIZE] {
    try_calculate_receipts_root(receipts)
        .expect("receipts root calculation must operate on previously validated canonical receipts")
}

/// Returns the canonical empty receipts root.
#[must_use]
pub fn empty_receipts_root() -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_RECEIPT_ROOT);
    update_u32(&mut hasher, 0);
    finalize_hash(hasher)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> Event {
        Event::new(1, vec![1, 2, 3]).expect("sample event must be valid")
    }

    fn sample_receipt() -> Receipt {
        let mut receipt =
            Receipt::success([1u8; HASH_SIZE], 100).expect("sample receipt must be valid");

        receipt
            .push_event(sample_event())
            .expect("event insertion must succeed");

        receipt
    }

    #[test]
    fn success_constructor_creates_valid_receipt() {
        let receipt = Receipt::success([7u8; HASH_SIZE], 55).expect("receipt must be valid");

        assert!(receipt.success);
        assert_eq!(receipt.error_code, None);
        assert!(receipt.validate().is_ok());
    }

    #[test]
    fn failure_constructor_creates_valid_receipt() {
        let receipt = Receipt::failure([7u8; HASH_SIZE], 55, 9).expect("receipt must be valid");

        assert!(!receipt.success);
        assert_eq!(receipt.error_code, Some(9));
        assert!(receipt.validate().is_ok());
    }

    #[test]
    fn validation_rejects_zero_transaction_hash() {
        let receipt = Receipt {
            transaction_hash: [0u8; HASH_SIZE],
            success: true,
            energy_used: 1,
            error_code: None,
            events: Vec::new(),
        };

        assert_eq!(receipt.validate(), Err(ReceiptError::ZeroTransactionHash));
    }

    #[test]
    fn validation_rejects_success_receipt_with_error_code() {
        let receipt = Receipt {
            transaction_hash: [1u8; HASH_SIZE],
            success: true,
            energy_used: 1,
            error_code: Some(9),
            events: Vec::new(),
        };

        assert_eq!(
            receipt.validate(),
            Err(ReceiptError::SuccessReceiptMustNotContainErrorCode)
        );
    }

    #[test]
    fn validation_rejects_failure_receipt_without_error_code() {
        let receipt = Receipt {
            transaction_hash: [1u8; HASH_SIZE],
            success: false,
            energy_used: 1,
            error_code: None,
            events: Vec::new(),
        };

        assert_eq!(
            receipt.validate(),
            Err(ReceiptError::FailureReceiptMustContainErrorCode)
        );
    }

    #[test]
    fn event_validation_rejects_oversized_payload() {
        let event = Event {
            event_type: 1,
            data: vec![0u8; MAX_EVENT_DATA_LEN + 1],
        };

        assert_eq!(event.validate(), Err(ReceiptError::EventDataTooLarge));
    }

    #[test]
    fn push_event_enforces_maximum_event_count() {
        let mut receipt = Receipt::success([1u8; HASH_SIZE], 1).expect("receipt must be valid");

        for _ in 0..MAX_EVENTS_PER_RECEIPT {
            receipt
                .push_event(Event::new(1, vec![1]).expect("event must be valid"))
                .expect("event insertion must succeed");
        }

        let result = receipt.push_event(Event::new(1, vec![1]).expect("event must be valid"));
        assert_eq!(result, Err(ReceiptError::TooManyEvents));
    }

    #[test]
    fn event_hash_is_deterministic() {
        let event = sample_event();
        assert_eq!(event.hash(), event.hash());
    }

    #[test]
    fn receipt_hash_is_deterministic() {
        let receipt = sample_receipt();

        let a = receipt.hash();
        let b = receipt.hash();

        assert_eq!(a, b);
    }

    #[test]
    fn receipt_hash_changes_with_events() {
        let r1 = sample_receipt();
        let mut r2 = sample_receipt();

        r2.push_event(Event::new(2, vec![9]).expect("event must be valid"))
            .expect("event insertion must succeed");

        assert_ne!(r1.hash(), r2.hash());
    }

    #[test]
    fn receipt_hash_changes_with_success_state() {
        let r1 = Receipt::success([1u8; HASH_SIZE], 100).expect("receipt must be valid");
        let r2 = Receipt::failure([1u8; HASH_SIZE], 100, 77).expect("receipt must be valid");

        assert_ne!(r1.hash(), r2.hash());
    }

    #[test]
    fn receipts_root_is_deterministic() {
        let r1 = sample_receipt();
        let r2 = sample_receipt();

        let root_a = calculate_receipts_root(&[r1.clone(), r2.clone()]);
        let root_b = calculate_receipts_root(&[r1, r2]);

        assert_eq!(root_a, root_b);
    }

    #[test]
    fn receipts_root_changes_when_order_changes() {
        let r1 = sample_receipt();

        let mut r2 = Receipt::success([2u8; HASH_SIZE], 100).expect("receipt must be valid");
        r2.push_event(Event::new(1, vec![1, 2, 3]).expect("event must be valid"))
            .expect("event insertion must succeed");

        let root_a = calculate_receipts_root(&[r1.clone(), r2.clone()]);
        let root_b = calculate_receipts_root(&[r2, r1]);

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn empty_receipts_root_matches_empty_calculation() {
        assert_eq!(empty_receipts_root(), calculate_receipts_root(&[]));
    }

    #[test]
    fn checked_len_rejects_usize_values_above_u32() {
        assert_eq!(
            checked_len((u32::MAX as usize) + 1),
            Err(ReceiptError::LengthOverflow)
        );
    }
}
