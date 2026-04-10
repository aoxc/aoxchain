// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Canonical algorithm-aware transaction envelope.
//!
//! This module unifies classical and post-quantum transaction transport shape
//! into one canonical envelope so admission, hashing, RPC, and SDK surfaces can
//! reason over a single signed-object grammar.

use core::fmt;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};

use crate::block::{Capability, TargetOutpost};
use crate::identity::pq_keys;
use crate::protocol::quantum::SignatureScheme;

use super::{MAX_TRANSACTION_PAYLOAD_BYTES, Transaction};

const ENVELOPE_SIGNING_DOMAIN: &[u8] = b"AOXC::TRANSACTION::ENVELOPE::SIGNING_PAYLOAD";
pub const ENVELOPE_SIGNING_FORMAT_VERSION: u8 = 1;
const LEGACY_CLASSIC_DOMAIN: &[u8] = b"AOXC::TRANSACTION::SIGNING_PAYLOAD";
const LEGACY_QUANTUM_DOMAIN: &[u8] = b"AOXC::TRANSACTION::QUANTUM::SIGNING_PAYLOAD";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionEnvelope {
    pub scheme_id: SignatureScheme,
    pub verification_key: Vec<u8>,
    pub nonce: u64,
    pub capability: Capability,
    pub target: TargetOutpost,
    pub payload: Vec<u8>,
    pub proof_bundle: Vec<u8>,
    pub profile_id: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EnvelopeError {
    EmptyVerificationKey,
    InvalidVerificationKey,
    InvalidProofBundle,
    InvalidNonce,
    EmptyPayload,
    PayloadTooLarge { size: usize, max: usize },
    ProfileIdMismatch { expected: u16, found: u16 },
    SignatureMismatch,
    LegacyFallbackRejected,
    UnsupportedScheme,
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVerificationKey => {
                f.write_str("envelope verification key must not be empty")
            }
            Self::InvalidVerificationKey => f.write_str("envelope verification key is invalid"),
            Self::InvalidProofBundle => f.write_str("envelope proof bundle is invalid"),
            Self::InvalidNonce => f.write_str("envelope nonce is invalid under active policy"),
            Self::EmptyPayload => f.write_str("envelope payload must not be empty"),
            Self::PayloadTooLarge { size, max } => write!(
                f,
                "envelope payload size {size} bytes exceeds maximum {max} bytes"
            ),
            Self::ProfileIdMismatch { expected, found } => write!(
                f,
                "envelope profile_id mismatch: expected profile_id {expected}, found {found}"
            ),
            Self::SignatureMismatch => {
                f.write_str("envelope proof bundle does not verify canonical signing message")
            }
            Self::LegacyFallbackRejected => {
                f.write_str("envelope signature is valid only under legacy fallback policy")
            }
            Self::UnsupportedScheme => f.write_str("envelope signature scheme is unsupported"),
        }
    }
}

impl std::error::Error for EnvelopeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvelopeVerificationPolicy {
    pub allow_legacy_signature_fallback: bool,
    pub expected_profile_id: Option<u16>,
}

impl EnvelopeVerificationPolicy {
    #[must_use]
    pub const fn permissive_default() -> Self {
        Self {
            allow_legacy_signature_fallback: true,
            expected_profile_id: None,
        }
    }

    #[must_use]
    pub const fn strict_for_profile(profile_id: u16) -> Self {
        Self {
            allow_legacy_signature_fallback: false,
            expected_profile_id: Some(profile_id),
        }
    }
}

impl TransactionEnvelope {
    pub fn validate(&self) -> Result<(), EnvelopeError> {
        self.validate_with_policy(EnvelopeVerificationPolicy::permissive_default())
    }

    pub fn validate_with_policy(
        &self,
        policy: EnvelopeVerificationPolicy,
    ) -> Result<(), EnvelopeError> {
        if self.verification_key.is_empty() {
            return Err(EnvelopeError::EmptyVerificationKey);
        }

        if self.nonce == 0 {
            return Err(EnvelopeError::InvalidNonce);
        }

        if self.payload.is_empty() {
            return Err(EnvelopeError::EmptyPayload);
        }

        if self.payload.len() > MAX_TRANSACTION_PAYLOAD_BYTES {
            return Err(EnvelopeError::PayloadTooLarge {
                size: self.payload.len(),
                max: MAX_TRANSACTION_PAYLOAD_BYTES,
            });
        }

        if let Some(expected_profile_id) = policy.expected_profile_id
            && expected_profile_id != self.profile_id
        {
            return Err(EnvelopeError::ProfileIdMismatch {
                expected: expected_profile_id,
                found: self.profile_id,
            });
        }

        self.verify_signature_with_policy(policy)
    }

    pub fn verify_signature(&self) -> Result<(), EnvelopeError> {
        self.verify_signature_with_policy(EnvelopeVerificationPolicy::permissive_default())
    }

    pub fn verify_signature_with_policy(
        &self,
        policy: EnvelopeVerificationPolicy,
    ) -> Result<(), EnvelopeError> {
        match self.scheme_id {
            SignatureScheme::Ed25519Legacy => {
                if self.verification_key.len() != 32 {
                    return Err(EnvelopeError::InvalidVerificationKey);
                }
                if self.proof_bundle.len() != 64 {
                    return Err(EnvelopeError::InvalidProofBundle);
                }

                let mut key = [0u8; 32];
                key.copy_from_slice(&self.verification_key);
                let mut signature = [0u8; 64];
                signature.copy_from_slice(&self.proof_bundle);

                let public_key = VerifyingKey::from_bytes(&key)
                    .map_err(|_| EnvelopeError::InvalidVerificationKey)?;
                let signature = Signature::from_bytes(&signature);

                if public_key
                    .verify(&self.signing_message(), &signature)
                    .is_ok()
                {
                    return Ok(());
                }

                if !policy.allow_legacy_signature_fallback {
                    return Err(EnvelopeError::LegacyFallbackRejected);
                }

                public_key
                    .verify(&self.legacy_classic_signing_message(), &signature)
                    .map_err(|_| EnvelopeError::SignatureMismatch)
            }
            SignatureScheme::MlDsa65 => {
                let public_key = pq_keys::public_key_from_bytes(&self.verification_key)
                    .map_err(|_| EnvelopeError::InvalidVerificationKey)?;
                let opened =
                    pq_keys::verify_message_domain_separated(&self.proof_bundle, &public_key)
                        .map_err(|_| EnvelopeError::InvalidProofBundle)?;

                if opened == self.signing_message() {
                    Ok(())
                } else if opened == self.legacy_quantum_signing_message() {
                    if policy.allow_legacy_signature_fallback {
                        Ok(())
                    } else {
                        Err(EnvelopeError::LegacyFallbackRejected)
                    }
                } else {
                    Err(EnvelopeError::SignatureMismatch)
                }
            }
            SignatureScheme::Dilithium3 | SignatureScheme::SphincsSha2128f => {
                Err(EnvelopeError::UnsupportedScheme)
            }
        }
    }

    pub fn signing_message(&self) -> Vec<u8> {
        let mut message = Vec::with_capacity(
            ENVELOPE_SIGNING_DOMAIN.len()
                + 1
                + 1
                + 4
                + self.verification_key.len()
                + 8
                + 1
                + 2
                + 4
                + self.payload.len()
                + 2,
        );

        message.extend_from_slice(ENVELOPE_SIGNING_DOMAIN);
        message.push(ENVELOPE_SIGNING_FORMAT_VERSION);
        message.push(self.scheme_id.code());
        message.extend_from_slice(&(self.verification_key.len() as u32).to_le_bytes());
        message.extend_from_slice(&self.verification_key);
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message.push(self.capability.code());
        message.extend_from_slice(&self.target.code().to_le_bytes());
        message.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        message.extend_from_slice(&self.payload);
        message.extend_from_slice(&self.profile_id.to_le_bytes());

        message
    }

    fn legacy_classic_signing_message(&self) -> Vec<u8> {
        let mut message = Vec::with_capacity(
            LEGACY_CLASSIC_DOMAIN.len()
                + 1
                + self.verification_key.len()
                + 8
                + 1
                + 2
                + 4
                + self.payload.len(),
        );

        message.extend_from_slice(LEGACY_CLASSIC_DOMAIN);
        message.push(1);
        message.extend_from_slice(&self.verification_key);
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message.push(self.capability.code());
        message.extend_from_slice(&self.target.code().to_le_bytes());
        message.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        message.extend_from_slice(&self.payload);
        message
    }

    fn legacy_quantum_signing_message(&self) -> Vec<u8> {
        let mut message = Vec::with_capacity(
            LEGACY_QUANTUM_DOMAIN.len() + 1 + 8 + 1 + 2 + 4 + self.payload.len(),
        );

        message.extend_from_slice(LEGACY_QUANTUM_DOMAIN);
        message.push(1);
        message.extend_from_slice(&self.nonce.to_le_bytes());
        message.push(self.capability.code());
        message.extend_from_slice(&self.target.code().to_le_bytes());
        message.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        message.extend_from_slice(&self.payload);
        message
    }
}

impl From<Transaction> for TransactionEnvelope {
    fn from(value: Transaction) -> Self {
        Self {
            scheme_id: SignatureScheme::Ed25519Legacy,
            verification_key: value.sender.to_vec(),
            nonce: value.nonce,
            capability: value.capability,
            target: value.target,
            payload: value.payload,
            proof_bundle: value.signature.to_vec(),
            profile_id: 0,
        }
    }
}

impl From<super::quantum::QuantumTransaction> for TransactionEnvelope {
    fn from(value: super::quantum::QuantumTransaction) -> Self {
        Self {
            scheme_id: SignatureScheme::MlDsa65,
            verification_key: value.sender_public_key,
            nonce: value.nonce,
            capability: value.capability,
            target: value.target,
            payload: value.payload,
            proof_bundle: value.signed_payload,
            profile_id: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn envelope_from_classic_transaction_verifies() {
        let signing_key = SigningKey::from_bytes(&[11u8; 32]);
        let sender = signing_key.verifying_key().to_bytes();

        let unsigned = Transaction {
            sender,
            nonce: 7,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload: vec![1, 2, 3],
            signature: [0u8; 64],
        };

        let mut signed = unsigned;
        signed.signature = signing_key.sign(&signed.signing_message()).to_bytes();

        let envelope: TransactionEnvelope = signed.into();
        assert_eq!(envelope.validate(), Ok(()));
    }

    #[test]
    fn envelope_from_quantum_transaction_verifies() {
        let (pk, sk) = pq_keys::generate_keypair();
        let payload = vec![9, 8, 7];
        let tx = super::super::quantum::QuantumTransaction::new(
            pq_keys::serialize_public_key(&pk),
            5,
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            payload.clone(),
            pq_keys::sign_message_domain_separated(
                &super::super::quantum::QuantumTransaction::canonical_signing_message(
                    5,
                    Capability::UserSigned,
                    TargetOutpost::EthMainnetGateway,
                    &payload,
                )
                .expect("quantum signing message must be valid"),
                &sk,
            ),
        )
        .expect("quantum transaction must be valid");

        let envelope: TransactionEnvelope = tx.into();
        assert_eq!(envelope.validate(), Ok(()));
    }

    #[test]
    fn strict_policy_rejects_legacy_fallback_signature() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let sender = signing_key.verifying_key().to_bytes();
        let legacy_signed_message = {
            let mut message = Vec::new();
            message.extend_from_slice(LEGACY_CLASSIC_DOMAIN);
            message.push(1);
            message.extend_from_slice(&sender);
            message.extend_from_slice(&9u64.to_le_bytes());
            message.push(Capability::UserSigned.code());
            message.extend_from_slice(&TargetOutpost::EthMainnetGateway.code().to_le_bytes());
            message.extend_from_slice(&(3u32).to_le_bytes());
            message.extend_from_slice(&[1u8, 2u8, 3u8]);
            message
        };

        let envelope = TransactionEnvelope {
            scheme_id: SignatureScheme::Ed25519Legacy,
            verification_key: sender.to_vec(),
            nonce: 9,
            capability: Capability::UserSigned,
            target: TargetOutpost::EthMainnetGateway,
            payload: vec![1, 2, 3],
            proof_bundle: signing_key.sign(&legacy_signed_message).to_bytes().to_vec(),
            profile_id: 2,
        };

        assert_eq!(
            envelope.validate_with_policy(EnvelopeVerificationPolicy::strict_for_profile(2)),
            Err(EnvelopeError::LegacyFallbackRejected)
        );
    }
}
