// AOXC MIT License
// Production-oriented protocol message envelope primitive.
// This component defines the canonical transport envelope used across AOXC
// modules and chain-family boundaries. The implementation is intentionally
// strict because any ambiguity at this layer may create replay, routing,
// hashing, or interoperability failures in downstream systems.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;


const PROTOCOL_MESSAGE_NAMESPACE: &[u8] = b"AOXC/PROTOCOL/MESSAGE_ENVELOPE";
const MESSAGE_ENVELOPE_HASH_VERSION: u8 = 1;
const ZERO32: [u8; 32] = [0u8; 32];
const ZERO16: [u8; 16] = [0u8; 16];

/// Upper bound selected to prevent unbounded type labels from becoming
/// an abuse vector for memory pressure, log amplification, or inconsistent
/// off-chain handling. The value is intentionally conservative.
const MAX_PAYLOAD_TYPE_LEN: usize = 64;

/// A protocol-level identifier for a first-class AOXC module.
///
/// The string representation of each variant is part of the canonical hash
/// domain. Any change to these mappings is therefore consensus-sensitive
/// for systems that depend on deterministic envelope hashing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ModuleId {
    RelayCore,
    Identity,
    Asset,
    Execution,
    Interop,
    Proof,
}

impl ModuleId {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelayCore => "relay_core",
            Self::Identity => "identity",
            Self::Asset => "asset",
            Self::Execution => "execution",
            Self::Interop => "interop",
            Self::Proof => "proof",
        }
    }
}

/// A constitutional sovereign domain recognized by AOXC.
///
/// This enum exists as a canonical vocabulary surface and is intentionally
/// separate from operational message routing. It should remain stable once
/// adopted by external clients and indexers.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SovereignRoot {
    Identity,
    Supply,
    Governance,
    Relay,
    Security,
    Settlement,
    Treasury,
}

impl SovereignRoot {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Supply => "supply",
            Self::Governance => "governance",
            Self::Relay => "relay",
            Self::Security => "security",
            Self::Settlement => "settlement",
            Self::Treasury => "treasury",
        }
    }
}

/// A normalized execution family used to classify heterogeneous chain targets.
///
/// The enum is intentionally protocol-facing rather than implementation-facing.
/// It supports deterministic routing, policy binding, and envelope hashing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChainFamily {
    Relay,
    Evm,
    Solana,
    Utxo,
    Ibc,
    Object,
    Wasm,
}

impl ChainFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Relay => "relay",
            Self::Evm => "evm",
            Self::Solana => "solana",
            Self::Utxo => "utxo",
            Self::Ibc => "ibc",
            Self::Object => "object",
            Self::Wasm => "wasm",
        }
    }
}

/// A coarse fee policy class attached to message execution.
///
/// This value is metadata. It does not by itself guarantee settlement,
/// prioritization, or economic sufficiency. Downstream executors and policy
/// engines must interpret it under their own verified rules.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeeClass {
    System,
    Standard,
    Priority,
}

impl FeeClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Standard => "standard",
            Self::Priority => "priority",
        }
    }
}

/// Validation failures for `MessageEnvelope`.
///
/// A typed error model is used instead of raw string slices so that production
/// callers may branch on exact failure semantics without string matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageEnvelopeError {
    SameSourceAndDestinationModule,
    EmptyPayloadType,
    PayloadTypeTooLong,
    PayloadTypeNotCanonical,
    ZeroNonce,
    ZeroPayloadHash,
    ZeroProofReference,
    ZeroReplayProtectionTag,
    ExpiryMustBeNonZeroWhenPresent,
}

impl fmt::Display for MessageEnvelopeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameSourceAndDestinationModule => {
                f.write_str("source_module and destination_module must differ")
            }
            Self::EmptyPayloadType => f.write_str("payload_type must not be empty"),
            Self::PayloadTypeTooLong => {
                f.write_str("payload_type exceeds the maximum canonical length")
            }
            Self::PayloadTypeNotCanonical => {
                f.write_str("payload_type must already be in canonical normalized form")
            }
            Self::ZeroNonce => f.write_str("nonce must not be zero"),
            Self::ZeroPayloadHash => f.write_str("payload_hash must not be zero"),
            Self::ZeroProofReference => {
                f.write_str("proof_reference must not be zero when present")
            }
            Self::ZeroReplayProtectionTag => f.write_str("replay_protection_tag must not be zero"),
            Self::ExpiryMustBeNonZeroWhenPresent => {
                f.write_str("expiry must be non-zero when present")
            }
        }
    }
}

impl std::error::Error for MessageEnvelopeError {}

/// Canonical protocol message envelope.
///
/// This structure transports routing and validation metadata for a payload,
/// not the payload bytes themselves. The `payload_hash` is expected to commit
/// to the actual payload under a separate, stable hashing specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageEnvelope {
    pub source_module: ModuleId,
    pub destination_module: ModuleId,
    pub source_chain_family: ChainFamily,
    pub target_chain_family: ChainFamily,
    pub nonce: u64,
    pub payload_type: String,
    pub payload_hash: [u8; 32],
    pub proof_reference: Option<[u8; 32]>,
    pub fee_class: FeeClass,
    pub expiry: Option<u64>,
    pub replay_protection_tag: [u8; 16],
}

impl MessageEnvelope {
    /// Constructs a new envelope with canonicalized `payload_type`.
    ///
    /// The constructor normalizes `payload_type` before validation so that
    /// callers receive a safe-by-default instance instead of needing to
    /// replicate canonicalization logic externally.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        source_module: ModuleId,
        destination_module: ModuleId,
        source_chain_family: ChainFamily,
        target_chain_family: ChainFamily,
        nonce: u64,
        payload_type: impl AsRef<str>,
        payload_hash: [u8; 32],
        proof_reference: Option<[u8; 32]>,
        fee_class: FeeClass,
        expiry: Option<u64>,
        replay_protection_tag: [u8; 16],
    ) -> Result<Self, MessageEnvelopeError> {
        let envelope = Self {
            source_module,
            destination_module,
            source_chain_family,
            target_chain_family,
            nonce,
            payload_type: canonicalize_payload_type(payload_type.as_ref())?.into_owned(),
            payload_hash,
            proof_reference,
            fee_class,
            expiry,
            replay_protection_tag,
        };

        envelope.validate()?;
        Ok(envelope)
    }

    /// Validates the envelope under strict canonical protocol rules.
    ///
    /// This method does not attempt to repair malformed inputs. Production
    /// systems should fail closed at this boundary and require callers to
    /// provide already canonical data or use the `new` constructor.
    pub fn validate(&self) -> Result<(), MessageEnvelopeError> {
        if self.source_module == self.destination_module {
            return Err(MessageEnvelopeError::SameSourceAndDestinationModule);
        }

        if self.nonce == 0 {
            return Err(MessageEnvelopeError::ZeroNonce);
        }

        if self.payload_hash == ZERO32 {
            return Err(MessageEnvelopeError::ZeroPayloadHash);
        }

        if self.replay_protection_tag == ZERO16 {
            return Err(MessageEnvelopeError::ZeroReplayProtectionTag);
        }

        if let Some(proof_reference) = self.proof_reference
            && proof_reference == ZERO32
        {
            return Err(MessageEnvelopeError::ZeroProofReference);
        }

        if let Some(expiry) = self.expiry
            && expiry == 0
        {
            return Err(MessageEnvelopeError::ExpiryMustBeNonZeroWhenPresent);
        }

        if self.payload_type.is_empty() {
            return Err(MessageEnvelopeError::EmptyPayloadType);
        }

        if self.payload_type.len() > MAX_PAYLOAD_TYPE_LEN {
            return Err(MessageEnvelopeError::PayloadTypeTooLong);
        }

        let canonical = canonicalize_payload_type(&self.payload_type)?;
        if canonical.as_ref() != self.payload_type {
            return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
        }

        Ok(())
    }

    /// Returns the deterministic protocol hash of the envelope.
    ///
    /// The function hashes field-presence markers for optional values so that:
    /// - `None` is distinct from `Some(0...)`
    /// - future decoders do not depend on implicit sentinel assumptions
    /// - canonical encoding semantics remain explicit
    ///
    /// The method panics only if the caller bypasses construction/validation
    /// discipline and invokes hashing on an invalid instance via unchecked
    /// mutation. In production usage, callers should construct through `new`
    /// or validate before hashing.
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        debug_assert!(self.validate().is_ok(), "hash() called on invalid envelope");

        let mut hasher = Hasher::new();
        hasher.update(PROTOCOL_MESSAGE_NAMESPACE);
        hasher.update(&[0x00, MESSAGE_ENVELOPE_HASH_VERSION]);

        hash_bytes(&mut hasher, self.source_module.as_str().as_bytes());
        hash_bytes(&mut hasher, self.destination_module.as_str().as_bytes());
        hash_bytes(&mut hasher, self.source_chain_family.as_str().as_bytes());
        hash_bytes(&mut hasher, self.target_chain_family.as_str().as_bytes());
        hash_bytes(&mut hasher, &self.nonce.to_le_bytes());
        hash_bytes(&mut hasher, self.payload_type.as_bytes());
        hash_bytes(&mut hasher, &self.payload_hash);

        match self.proof_reference {
            Some(value) => {
                hash_bytes(&mut hasher, &[1u8]);
                hash_bytes(&mut hasher, &value);
            }
            None => {
                hash_bytes(&mut hasher, &[0u8]);
            }
        }

        hash_bytes(&mut hasher, self.fee_class.as_str().as_bytes());

        match self.expiry {
            Some(value) => {
                hash_bytes(&mut hasher, &[1u8]);
                hash_bytes(&mut hasher, &value.to_le_bytes());
            }
            None => {
                hash_bytes(&mut hasher, &[0u8]);
            }
        }

        hash_bytes(&mut hasher, &self.replay_protection_tag);

        *hasher.finalize().as_bytes()
    }

    /// Returns `true` when an expiry is present and the supplied timestamp
    /// strictly exceeds the permitted execution boundary.
    ///
    /// The timestamp unit must match the protocol-wide convention enforced
    /// by the caller, for example Unix seconds.
    #[must_use]
    pub fn is_expired_at(&self, now: u64) -> bool {
        match self.expiry {
            Some(expiry) => now > expiry,
            None => false,
        }
    }

    /// Returns the canonical payload type string.
    #[must_use]
    pub fn payload_type(&self) -> &str {
        &self.payload_type
    }
}

/// Hashes a byte slice using an explicit length prefix and field separator.
///
/// This avoids ambiguity that could arise from simple concatenation, especially
/// if future domains introduce new field shapes or variable-length values.
fn hash_bytes(hasher: &mut Hasher, bytes: &[u8]) {
    let len = bytes.len() as u64;
    hasher.update(&len.to_le_bytes());
    hasher.update(bytes);
    hasher.update(&[0xFF]);
}

/// Canonicalizes a payload type label.
///
/// Canonical form requirements:
/// - ASCII lowercase only
/// - leading and trailing whitespace removed
/// - internal whitespace not permitted
/// - segments separated by '.'
/// - each segment contains only `[a-z0-9_]`
/// - no empty segments
///
/// Examples of valid values:
/// - `bridge.commitment`
/// - `identity.key_rotation`
/// - `proof.batch_v2`
pub(crate) fn canonicalize_payload_type(input: &str) -> Result<Cow<'_, str>, MessageEnvelopeError> {
    let trimmed = input.trim();

    if trimmed.is_empty() {
        return Err(MessageEnvelopeError::EmptyPayloadType);
    }

    if trimmed.len() > MAX_PAYLOAD_TYPE_LEN {
        return Err(MessageEnvelopeError::PayloadTypeTooLong);
    }

    if trimmed.as_bytes().iter().any(u8::is_ascii_whitespace) {
        return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
    }

    if !trimmed.is_ascii() {
        return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
    }

    let mut normalized = String::with_capacity(trimmed.len());
    let mut previous_was_dot = false;

    for (index, ch) in trimmed.chars().enumerate() {
        let canonical_ch = ch.to_ascii_lowercase();

        let is_valid = canonical_ch.is_ascii_lowercase()
            || canonical_ch.is_ascii_digit()
            || canonical_ch == '_'
            || canonical_ch == '.';

        if !is_valid {
            return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
        }

        if canonical_ch == '.' {
            if index == 0 || previous_was_dot {
                return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
            }
            previous_was_dot = true;
        } else {
            previous_was_dot = false;
        }

        normalized.push(canonical_ch);
    }

    if normalized.ends_with('.') {
        return Err(MessageEnvelopeError::PayloadTypeNotCanonical);
    }

    if normalized == trimmed {
        return Ok(Cow::Borrowed(trimmed));
    }

    Ok(Cow::Owned(normalized))
}

#[must_use]
pub const fn canonical_modules() -> [ModuleId; 6] {
    [
        ModuleId::RelayCore,
        ModuleId::Identity,
        ModuleId::Asset,
        ModuleId::Execution,
        ModuleId::Interop,
        ModuleId::Proof,
    ]
}

#[must_use]
pub const fn canonical_chain_families() -> [ChainFamily; 5] {
    [
        ChainFamily::Evm,
        ChainFamily::Solana,
        ChainFamily::Utxo,
        ChainFamily::Ibc,
        ChainFamily::Object,
    ]
}

#[must_use]
pub const fn canonical_message_envelope_fields() -> [&'static str; 11] {
    [
        "sourceModule",
        "destinationModule",
        "sourceChainFamily",
        "targetChainFamily",
        "nonce",
        "payloadType",
        "payloadHash",
        "proofReference",
        "feeClass",
        "expiry",
        "replayProtectionTag",
    ]
}

#[must_use]
pub const fn canonical_sovereign_roots() -> [SovereignRoot; 7] {
    [
        SovereignRoot::Identity,
        SovereignRoot::Supply,
        SovereignRoot::Governance,
        SovereignRoot::Relay,
        SovereignRoot::Security,
        SovereignRoot::Settlement,
        SovereignRoot::Treasury,
    ]
}
