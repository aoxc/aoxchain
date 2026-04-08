// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Quantum-native transaction primitive.
//!
//! This module introduces a post-quantum signed transaction object that uses
//! AOXC domain-separated ML-DSA signing through `identity::pq_keys`.

use core::fmt;

use crate::block::{BlockError, Capability, TargetOutpost, Task};
use crate::identity::pq_keys;
use crate::protocol::quantum::SignatureScheme;

use super::MAX_TRANSACTION_PAYLOAD_BYTES;

/// Canonical signing-message format version for quantum transactions.
pub const QUANTUM_TX_SIGNING_FORMAT_VERSION: u8 = 1;
/// Canonical hash format version for quantum transaction identifiers.
pub const QUANTUM_TX_HASH_FORMAT_VERSION: u8 = 1;

const QUANTUM_TRANSACTION_SIGNING_DOMAIN: &[u8] = b"AOXC::TRANSACTION::QUANTUM::SIGNING_PAYLOAD";
const QUANTUM_TRANSACTION_INTENT_HASH_DOMAIN: &[u8] = b"AOXC::TRANSACTION::QUANTUM::INTENT_ID";
const QUANTUM_TRANSACTION_HASH_DOMAIN: &[u8] = b"AOXC::TRANSACTION::QUANTUM::TX_ID";

/// Canonical error surface for quantum transaction validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum QuantumTransactionError {
    InvalidSenderPublicKey,
    InvalidSignedPayload,
    InvalidNonce,
    PayloadTooLarge { size: usize, max: usize },
    EmptyPayload,
    SigningMessageMismatch,
    TaskConversionFailed(BlockError),
}

impl QuantumTransactionError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidSenderPublicKey => "QTX_INVALID_SENDER_PUBLIC_KEY",
            Self::InvalidSignedPayload => "QTX_INVALID_SIGNED_PAYLOAD",
            Self::InvalidNonce => "QTX_INVALID_NONCE",
            Self::PayloadTooLarge { .. } => "QTX_PAYLOAD_TOO_LARGE",
            Self::EmptyPayload => "QTX_EMPTY_PAYLOAD",
            Self::SigningMessageMismatch => "QTX_SIGNING_MESSAGE_MISMATCH",
            Self::TaskConversionFailed(_) => "QTX_TASK_CONVERSION_FAILED",
        }
    }
}

impl fmt::Display for QuantumTransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSenderPublicKey => {
                f.write_str("quantum transaction sender public key is invalid")
            }
            Self::InvalidSignedPayload => {
                f.write_str("quantum transaction signature verification failed")
            }
            Self::InvalidNonce => f.write_str("quantum transaction nonce is invalid"),
            Self::PayloadTooLarge { size, max } => write!(
                f,
                "quantum transaction payload size {} bytes exceeds maximum {} bytes",
                size, max
            ),
            Self::EmptyPayload => f.write_str("quantum transaction payload must not be empty"),
            Self::SigningMessageMismatch => {
                f.write_str("quantum transaction signed payload does not match canonical message")
            }
            Self::TaskConversionFailed(err) => {
                write!(f, "quantum transaction-to-task conversion failed: {err}")
            }
        }
    }
}

impl std::error::Error for QuantumTransactionError {}

/// Canonical quantum-native AOXC transaction object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantumTransaction {
    /// Serialized ML-DSA verification key bytes.
    pub sender_public_key: Vec<u8>,
    /// Replay-protection counter.
    pub nonce: u64,
    /// Authorization class.
    pub capability: Capability,
    /// Logical routing destination.
    pub target: TargetOutpost,
    /// Opaque routed command payload.
    pub payload: Vec<u8>,
    /// Domain-separated signed payload returned by `pq_keys`.
    pub signed_payload: Vec<u8>,
}

impl QuantumTransaction {
    /// Constructs a new quantum transaction and validates structural invariants.
    pub fn new(
        sender_public_key: Vec<u8>,
        nonce: u64,
        capability: Capability,
        target: TargetOutpost,
        payload: Vec<u8>,
        signed_payload: Vec<u8>,
    ) -> Result<Self, QuantumTransactionError> {
        let tx = Self {
            sender_public_key,
            nonce,
            capability,
            target,
            payload,
            signed_payload,
        };

        tx.validate()?;
        Ok(tx)
    }

    /// Validates transaction-local invariants.
    pub fn validate(&self) -> Result<(), QuantumTransactionError> {
        if self.nonce == 0 {
            return Err(QuantumTransactionError::InvalidNonce);
        }

        if self.payload.is_empty() {
            return Err(QuantumTransactionError::EmptyPayload);
        }

        if self.payload.len() > MAX_TRANSACTION_PAYLOAD_BYTES {
            return Err(QuantumTransactionError::PayloadTooLarge {
                size: self.payload.len(),
                max: MAX_TRANSACTION_PAYLOAD_BYTES,
            });
        }

        if pq_keys::public_key_from_bytes(&self.sender_public_key).is_err() {
            return Err(QuantumTransactionError::InvalidSenderPublicKey);
        }
        self.verify_signature()
    }

    /// Verifies the signed payload against canonical signing-message bytes.
    pub fn verify_signature(&self) -> Result<(), QuantumTransactionError> {
        let public_key = pq_keys::public_key_from_bytes(&self.sender_public_key)
            .map_err(|_| QuantumTransactionError::InvalidSenderPublicKey)?;
        let opened = pq_keys::verify_message_domain_separated(&self.signed_payload, &public_key)
            .map_err(|_| QuantumTransactionError::InvalidSignedPayload)?;

        let expected = self.signing_message()?;
        if opened != expected {
            return Err(QuantumTransactionError::SigningMessageMismatch);
        }

        Ok(())
    }

    /// Returns the signature scheme used by this transaction type.
    #[must_use]
    pub const fn signature_scheme(&self) -> SignatureScheme {
        SignatureScheme::MlDsa65
    }

    /// Returns a deterministic unsigned intent identifier for this transaction.
    #[must_use]
    pub fn intent_id(&self) -> [u8; 32] {
        self.hash_quantum_transaction(QUANTUM_TRANSACTION_INTENT_HASH_DOMAIN, false)
    }

    /// Returns a deterministic signed transaction identifier for this transaction.
    #[must_use]
    pub fn tx_id(&self) -> [u8; 32] {
        self.hash_quantum_transaction(QUANTUM_TRANSACTION_HASH_DOMAIN, true)
    }

    /// Validates nonce using a caller-supplied policy hook.
    pub fn validate_nonce_with<F>(&self, is_valid_nonce: F) -> Result<(), QuantumTransactionError>
    where
        F: FnOnce(u64) -> bool,
    {
        if is_valid_nonce(self.nonce) {
            Ok(())
        } else {
            Err(QuantumTransactionError::InvalidNonce)
        }
    }

    /// Returns the canonical signing message bytes.
    pub fn signing_message(&self) -> Result<Vec<u8>, QuantumTransactionError> {
        Self::canonical_signing_message(self.nonce, self.capability, self.target, &self.payload)
    }

    /// Builds canonical signing message bytes from transaction fields.
    pub fn canonical_signing_message(
        nonce: u64,
        capability: Capability,
        target: TargetOutpost,
        payload: &[u8],
    ) -> Result<Vec<u8>, QuantumTransactionError> {
        let payload_len =
            u32::try_from(payload.len()).map_err(|_| QuantumTransactionError::PayloadTooLarge {
                size: payload.len(),
                max: MAX_TRANSACTION_PAYLOAD_BYTES,
            })?;

        let mut message = Vec::with_capacity(
            QUANTUM_TRANSACTION_SIGNING_DOMAIN.len() + 1 + 8 + 1 + 2 + 4 + payload.len(),
        );

        message.extend_from_slice(QUANTUM_TRANSACTION_SIGNING_DOMAIN);
        message.push(QUANTUM_TX_SIGNING_FORMAT_VERSION);
        message.extend_from_slice(&nonce.to_le_bytes());
        message.push(capability.code());
        message.extend_from_slice(&target.code().to_le_bytes());
        message.extend_from_slice(&payload_len.to_le_bytes());
        message.extend_from_slice(payload);

        Ok(message)
    }

    fn hash_quantum_transaction(&self, domain: &[u8], include_signature: bool) -> [u8; 32] {
        let signing_message = self
            .signing_message()
            .expect("canonical quantum signing message must be representable for valid payloads");

        let mut hasher = blake3::Hasher::new();
        hasher.update(domain);
        hasher.update(&[QUANTUM_TX_HASH_FORMAT_VERSION]);
        hasher.update(&u32::to_le_bytes(self.sender_public_key.len() as u32));
        hasher.update(&self.sender_public_key);
        hasher.update(&u32::to_le_bytes(signing_message.len() as u32));
        hasher.update(&signing_message);

        if include_signature {
            hasher.update(&u32::to_le_bytes(self.signed_payload.len() as u32));
            hasher.update(&self.signed_payload);
        }

        *hasher.finalize().as_bytes()
    }

    /// Converts this quantum transaction into a block-domain task.
    pub fn to_task(&self) -> Result<Task, QuantumTransactionError> {
        self.validate()?;
        Task::new(
            self.tx_id(),
            self.capability,
            self.target,
            self.payload.clone(),
        )
        .map_err(QuantumTransactionError::TaskConversionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signed_quantum_tx(payload: Vec<u8>, nonce: u64) -> QuantumTransaction {
        let (pk, sk) = pq_keys::generate_keypair();
        let message = QuantumTransaction::canonical_signing_message(
            nonce,
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            &payload,
        )
        .expect("signing message must be valid");

        let signed_payload = pq_keys::sign_message_domain_separated(&message, &sk);

        QuantumTransaction {
            sender_public_key: pq_keys::serialize_public_key(&pk),
            nonce,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload,
            signed_payload,
        }
    }

    #[test]
    fn valid_quantum_transaction_signature_verifies() {
        let tx = signed_quantum_tx(vec![1, 2, 3, 4], 1);
        assert_eq!(tx.validate(), Ok(()));
        assert_eq!(tx.verify_signature(), Ok(()));
    }

    #[test]
    fn modified_payload_breaks_quantum_signature() {
        let mut tx = signed_quantum_tx(vec![1, 2, 3, 4], 1);
        tx.payload.push(9);

        assert_eq!(
            tx.verify_signature(),
            Err(QuantumTransactionError::SigningMessageMismatch)
        );
    }

    #[test]
    fn zero_nonce_is_rejected() {
        let tx = signed_quantum_tx(vec![1, 2, 3], 0);
        assert_eq!(tx.validate(), Err(QuantumTransactionError::InvalidNonce));
    }

    #[test]
    fn nonce_policy_hook_rejects_invalid_nonce() {
        let tx = signed_quantum_tx(vec![1, 2, 3], 42);
        assert_eq!(
            tx.validate_nonce_with(|nonce| nonce == 41),
            Err(QuantumTransactionError::InvalidNonce)
        );
    }

    #[test]
    fn quantum_transaction_ids_are_stable_and_signature_sensitive() {
        let tx = signed_quantum_tx(vec![1, 2, 3], 42);
        let cloned = tx.clone();
        assert_eq!(tx.intent_id(), cloned.intent_id());
        assert_eq!(tx.tx_id(), cloned.tx_id());

        let mut tampered_signature = tx.clone();
        tampered_signature.signed_payload.push(0xAB);

        assert_eq!(tx.intent_id(), tampered_signature.intent_id());
        assert_ne!(tx.tx_id(), tampered_signature.tx_id());
    }
}
