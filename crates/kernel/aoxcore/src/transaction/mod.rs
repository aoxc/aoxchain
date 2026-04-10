// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/transaction/src/mod.rs
//!
//! AOXC Transaction Domain Module.
//!
//! This module defines the canonical signed transaction model used to submit
//! routing-oriented execution intents into the AOXC / AOVM pipeline.
//!
//! Design objectives:
//! - Deterministic and minimal transaction representation
//! - Canonical Ed25519 signing semantics
//! - Clear separation between structural validation and stateful nonce policy
//! - Direct compatibility with the block-domain task model
//! - Clean interoperability with transaction hashing and pool admission
//! - Explicit fail-closed guards against malformed sentinel states

pub mod envelope;
pub mod hash;
pub mod pool;
pub mod quantum;

pub use envelope::{ENVELOPE_SIGNING_FORMAT_VERSION, EnvelopeError, TransactionEnvelope};
pub use hash::{
    HASH_FORMAT_VERSION, HASH_SIZE, TransactionHashError, ZERO_HASH, calculate_transaction_root,
    compute_hash, empty_transaction_root, hash_internal_node, hash_signing_payload,
    hash_transaction, hash_transaction_intent, hash_transaction_leaf,
    try_calculate_transaction_root, try_compute_hash, try_hash_signing_payload,
    try_hash_transaction, try_hash_transaction_intent, try_hash_transaction_leaf,
};
pub use pool::{
    SenderId, TransactionId, TransactionPool, TransactionPoolConfig, TransactionPoolError,
};
pub use quantum::{
    QUANTUM_TX_HASH_FORMAT_VERSION, QUANTUM_TX_SIGNING_FORMAT_VERSION, QuantumTransaction,
    QuantumTransactionError,
};

use core::fmt;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::block::{BlockError, Capability, TargetOutpost, Task};

/// Maximum permitted transaction payload size in bytes.
///
/// This bound protects memory usage, networking cost, and signature-verification
/// surfaces from oversized opaque command payloads.
pub const MAX_TRANSACTION_PAYLOAD_BYTES: usize = 64 * 1024;

/// Canonical signing-message format version.
///
/// This value must be incremented if the byte-level signing message layout
/// changes in a backward-incompatible way.
pub const TRANSACTION_SIGNING_FORMAT_VERSION: u8 = 1;

/// Canonical domain separator for transaction signing.
///
/// The signing payload must remain domain-separated from all other message
/// classes in the protocol.
const TRANSACTION_SIGNING_DOMAIN: &[u8] = b"AOXC::TRANSACTION::SIGNING_PAYLOAD";

const ZERO_SENDER: [u8; 32] = [0u8; 32];
const ZERO_SIGNATURE: [u8; 64] = [0u8; 64];

/// Canonical transaction-domain error type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransactionError {
    /// The sender public key is malformed, unsupported, or trivially zeroed.
    InvalidSenderKey,

    /// The transaction signature is structurally invalid or trivially zeroed.
    InvalidSignature,

    /// The nonce is invalid under an external caller-supplied policy.
    InvalidNonce,

    /// The payload exceeds the configured size limit.
    PayloadTooLarge {
        /// Observed payload size in bytes.
        size: usize,
        /// Maximum allowed payload size in bytes.
        max: usize,
    },

    /// The payload is empty and therefore not meaningful as a routed command.
    EmptyPayload,

    /// Canonical transaction hashing/signing encoding failed.
    HashEncodingFailed(hash::TransactionHashError),

    /// Conversion into a block-domain task failed.
    TaskConversionFailed(BlockError),
}

impl TransactionError {
    /// Returns a stable symbolic code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidSenderKey => "TX_INVALID_SENDER_KEY",
            Self::InvalidSignature => "TX_INVALID_SIGNATURE",
            Self::InvalidNonce => "TX_INVALID_NONCE",
            Self::PayloadTooLarge { .. } => "TX_PAYLOAD_TOO_LARGE",
            Self::EmptyPayload => "TX_EMPTY_PAYLOAD",
            Self::HashEncodingFailed(_) => "TX_HASH_ENCODING_FAILED",
            Self::TaskConversionFailed(_) => "TX_TASK_CONVERSION_FAILED",
        }
    }

    /// Returns `true` if the error represents a local invariant violation.
    #[must_use]
    pub const fn is_invariant_violation(self) -> bool {
        match self {
            Self::InvalidSenderKey | Self::InvalidSignature => false,
            Self::InvalidNonce
            | Self::PayloadTooLarge { .. }
            | Self::EmptyPayload
            | Self::HashEncodingFailed(_)
            | Self::TaskConversionFailed(_) => true,
        }
    }
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSenderKey => write!(
                f,
                "transaction validation failed: sender public key is malformed, unsupported, or zeroed"
            ),
            Self::InvalidSignature => write!(
                f,
                "transaction validation failed: signature does not match the canonical signing payload or is structurally invalid"
            ),
            Self::InvalidNonce => write!(
                f,
                "transaction validation failed: nonce is invalid under the active policy"
            ),
            Self::PayloadTooLarge { size, max } => write!(
                f,
                "transaction validation failed: payload size {} bytes exceeds maximum allowed size {} bytes",
                size, max
            ),
            Self::EmptyPayload => write!(
                f,
                "transaction validation failed: payload must not be empty"
            ),
            Self::HashEncodingFailed(err) => write!(
                f,
                "transaction canonical encoding failed during hashing/signing: {}",
                err
            ),
            Self::TaskConversionFailed(err) => {
                write!(f, "transaction-to-task conversion failed: {}", err)
            }
        }
    }
}

impl std::error::Error for TransactionError {}

impl From<BlockError> for TransactionError {
    fn from(value: BlockError) -> Self {
        Self::TaskConversionFailed(value)
    }
}

impl From<hash::TransactionHashError> for TransactionError {
    fn from(value: hash::TransactionHashError) -> Self {
        Self::HashEncodingFailed(value)
    }
}

/// Canonical AOXC transaction object.
///
/// A transaction is a signed routing intent carrying:
/// - sender identity
/// - replay-protection nonce
/// - authorization class
/// - logical destination
/// - opaque payload
/// - sender signature over the canonical signing payload
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Sender public key.
    pub sender: [u8; 32],

    /// Replay-protection counter.
    pub nonce: u64,

    /// Authorization class.
    pub capability: Capability,

    /// Logical routing destination.
    pub target: TargetOutpost,

    /// Opaque routed command payload.
    pub payload: Vec<u8>,

    /// Ed25519 signature over the canonical signing payload.
    pub signature: [u8; 64],
}

impl Transaction {
    /// Creates a new transaction and validates its structural invariants.
    pub fn new(
        sender: [u8; 32],
        nonce: u64,
        capability: Capability,
        target: TargetOutpost,
        payload: Vec<u8>,
        signature: [u8; 64],
    ) -> Result<Self, TransactionError> {
        let tx = Self {
            sender,
            nonce,
            capability,
            target,
            payload,
            signature,
        };

        tx.validate()?;
        Ok(tx)
    }

    /// Validates transaction-local invariants.
    ///
    /// This method does not perform stateful nonce checks. Nonce correctness
    /// depends on external chain or mempool state and must therefore be
    /// enforced by the caller.
    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.sender == ZERO_SENDER {
            return Err(TransactionError::InvalidSenderKey);
        }

        if self.signature == ZERO_SIGNATURE {
            return Err(TransactionError::InvalidSignature);
        }

        if self.payload.is_empty() {
            return Err(TransactionError::EmptyPayload);
        }

        if self.payload.len() > MAX_TRANSACTION_PAYLOAD_BYTES {
            return Err(TransactionError::PayloadTooLarge {
                size: self.payload.len(),
                max: MAX_TRANSACTION_PAYLOAD_BYTES,
            });
        }

        let _ = self.verifying_key()?;

        Ok(())
    }

    /// Returns the sender verifying key after canonical decoding.
    pub fn verifying_key(&self) -> Result<VerifyingKey, TransactionError> {
        VerifyingKey::from_bytes(&self.sender).map_err(|_| TransactionError::InvalidSenderKey)
    }

    /// Returns the payload size in bytes.
    #[must_use]
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }

    /// Returns `true` when the payload is empty.
    #[must_use]
    pub fn is_empty_payload(&self) -> bool {
        self.payload.is_empty()
    }

    /// Builds the canonical signing payload used for Ed25519 verification.
    ///
    /// Encoding layout:
    /// - signing domain
    /// - signing format version
    /// - sender public key
    /// - nonce
    /// - capability code
    /// - target code
    /// - payload length
    /// - payload bytes
    pub fn try_signing_message(&self) -> Result<Vec<u8>, TransactionError> {
        let mut message = Vec::with_capacity(
            TRANSACTION_SIGNING_DOMAIN.len() + 1 + 32 + 8 + 1 + 2 + 4 + self.payload.len(),
        );

        message.extend_from_slice(TRANSACTION_SIGNING_DOMAIN);
        message.push(TRANSACTION_SIGNING_FORMAT_VERSION);
        message.extend_from_slice(&self.sender);
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message.push(self.capability.code());
        message.extend_from_slice(&self.target.code().to_le_bytes());

        let payload_len = u32::try_from(self.payload.len())
            .map_err(|_| hash::TransactionHashError::LengthOverflow)
            .map_err(TransactionError::from)?;
        message.extend_from_slice(&payload_len.to_le_bytes());
        message.extend_from_slice(&self.payload);

        Ok(message)
    }

    /// Returns the canonical signing message and panics only if a previously
    /// validated payload violated canonical bounds through unchecked mutation.
    #[must_use]
    pub fn signing_message(&self) -> Vec<u8> {
        self.try_signing_message().expect(
            "transaction signing message construction must operate on canonical payload bounds",
        )
    }

    /// Verifies the transaction signature against the canonical signing payload.
    pub fn verify_signature(&self) -> Result<(), TransactionError> {
        self.validate()?;

        let public_key = self.verifying_key()?;
        let signature = Signature::from_bytes(&self.signature);
        let message = self.try_signing_message()?;

        public_key
            .verify(&message, &signature)
            .map_err(|_| TransactionError::InvalidSignature)
    }

    /// Validates the nonce using a caller-supplied policy hook.
    ///
    /// This method exists because nonce validity is inherently stateful.
    pub fn validate_nonce_with<F>(&self, is_valid_nonce: F) -> Result<(), TransactionError>
    where
        F: FnOnce(u64) -> bool,
    {
        if is_valid_nonce(self.nonce) {
            Ok(())
        } else {
            Err(TransactionError::InvalidNonce)
        }
    }

    /// Returns the canonical unsigned intent identifier.
    pub fn try_intent_id(&self) -> Result<[u8; 32], TransactionError> {
        hash::try_hash_transaction_intent(self).map_err(TransactionError::from)
    }

    /// Returns the canonical unsigned intent identifier.
    #[must_use]
    pub fn intent_id(&self) -> [u8; 32] {
        self.try_intent_id()
            .expect("transaction intent hashing must operate on canonical transaction bounds")
    }

    /// Returns the canonical signed transaction identifier.
    pub fn try_tx_id(&self) -> Result<[u8; 32], TransactionError> {
        hash::try_hash_transaction(self).map_err(TransactionError::from)
    }

    /// Returns the canonical signed transaction identifier.
    #[must_use]
    pub fn tx_id(&self) -> [u8; 32] {
        self.try_tx_id()
            .expect("transaction id hashing must operate on canonical transaction bounds")
    }

    /// Converts this transaction into a block-domain task.
    ///
    /// The task identifier is derived from the signed transaction hash to bind
    /// the resulting task to the sealed transaction object.
    pub fn to_task(&self) -> Result<Task, TransactionError> {
        Task::new(
            self.try_tx_id()?,
            self.capability,
            self.target,
            self.payload.clone(),
        )
        .map_err(TransactionError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn signing_key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn signed_transaction(payload: Vec<u8>, nonce: u64) -> Transaction {
        let signing_key = signing_key(7);
        let sender = signing_key.verifying_key().to_bytes();

        let unsigned = Transaction {
            sender,
            nonce,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload,
            signature: [0u8; 64],
        };

        let signature = signing_key.sign(&unsigned.signing_message()).to_bytes();

        Transaction {
            signature,
            ..unsigned
        }
    }

    #[test]
    fn valid_transaction_signature_verifies() {
        let tx = signed_transaction(vec![1, 2, 3, 4], 1);
        assert_eq!(tx.verify_signature(), Ok(()));
        assert!(tx.try_signing_message().is_ok());
        assert!(tx.try_intent_id().is_ok());
        assert!(tx.try_tx_id().is_ok());
    }

    #[test]
    fn modified_payload_breaks_signature() {
        let mut tx = signed_transaction(vec![1, 2, 3, 4], 1);
        tx.payload.push(9);

        assert_eq!(
            tx.verify_signature(),
            Err(TransactionError::InvalidSignature)
        );
    }

    #[test]
    fn empty_payload_is_rejected() {
        let tx = signed_transaction(Vec::new(), 1);
        assert_eq!(tx.validate(), Err(TransactionError::EmptyPayload));
    }

    #[test]
    fn zero_sender_is_rejected() {
        let tx = Transaction {
            sender: [0u8; 32],
            nonce: 1,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload: vec![1],
            signature: [1u8; 64],
        };

        assert_eq!(tx.validate(), Err(TransactionError::InvalidSenderKey));
    }

    #[test]
    fn zero_signature_is_rejected() {
        let signing_key = signing_key(7);
        let tx = Transaction {
            sender: signing_key.verifying_key().to_bytes(),
            nonce: 1,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload: vec![1],
            signature: [0u8; 64],
        };

        assert_eq!(tx.validate(), Err(TransactionError::InvalidSignature));
    }

    #[test]
    fn nonce_policy_hook_accepts_valid_nonce() {
        let tx = signed_transaction(vec![1, 2, 3], 42);
        assert_eq!(tx.validate_nonce_with(|nonce| nonce == 42), Ok(()));
    }

    #[test]
    fn nonce_policy_hook_rejects_invalid_nonce() {
        let tx = signed_transaction(vec![1, 2, 3], 42);
        assert_eq!(
            tx.validate_nonce_with(|nonce| nonce == 41),
            Err(TransactionError::InvalidNonce)
        );
    }

    #[test]
    fn transaction_converts_to_task() {
        let tx = signed_transaction(vec![1, 2, 3], 5);
        let task = tx
            .to_task()
            .expect("transaction must convert to a valid task");

        assert_eq!(task.task_id, tx.tx_id());
        assert_eq!(task.capability, tx.capability);
        assert_eq!(task.target_outpost, tx.target);
        assert_eq!(task.payload, tx.payload);
    }
}
