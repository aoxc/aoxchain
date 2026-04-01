// AOXC MIT License
// Kernel-facing interoperability boundary types.
// This module intentionally models classification and policy boundaries only.
// It does not implement bridging logic, verifier internals, or relayer behavior.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical foreign chain profile key used by kernel routing and policy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ChainProfileId(String);

impl ChainProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, KernelInteropError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(KernelInteropError::EmptyChainProfileId);
        }
        if id.len() > 64 {
            return Err(KernelInteropError::ChainProfileIdTooLong);
        }
        if !id.is_ascii() {
            return Err(KernelInteropError::ChainProfileIdNotAscii);
        }
        Ok(Self(id))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Canonical proof taxonomy understood by kernel policy and dispatch boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProofType {
    HeaderCommitment,
    MerkleInclusion,
    SignatureQuorum,
    LightClientState,
    ValidityProof,
    FraudProof,
}

/// Compatibility class used by kernel policy to reason about foreign execution families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainCompatibilityClass {
    EvmLike,
    UtxoLike,
    AccountBftLike,
    ObjectCapabilityLike,
    IbcLike,
    CustomConstrained,
}

/// Finality model classes that govern settlement safety decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FinalityClass {
    DeterministicImmediate,
    ProbabilisticConfirmations,
    Checkpointed,
    EpochQuorum,
    OptimisticChallengeWindow,
}

/// Authority domain boundaries used for universal identity mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthorityDomain {
    NativeValidatorSet,
    ContractGoverned,
    MultiSigCommittee,
    ExternalProtocolCouncil,
}

/// Canonical settlement direction controlled by kernel policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettlementDirection {
    Inbound,
    Outbound,
}

/// Kernel-owned identifier for a verified counterpart relationship.
///
/// A counterpart relationship links AOXC-local coordination state to a remote
/// canonical program/contract that remains authoritative on its origin chain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct CounterpartId(String);

impl CounterpartId {
    pub fn new(id: impl Into<String>) -> Result<Self, KernelInteropError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(KernelInteropError::EmptyCounterpartId);
        }
        if id.len() > 96 {
            return Err(KernelInteropError::CounterpartIdTooLong);
        }
        if !id.is_ascii() {
            return Err(KernelInteropError::CounterpartIdNotAscii);
        }
        Ok(Self(id))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Remote entry gate identifier for paired interaction surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct RemoteGateId(String);

impl RemoteGateId {
    pub fn new(id: impl Into<String>) -> Result<Self, KernelInteropError> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(KernelInteropError::EmptyRemoteGateId);
        }
        if id.len() > 96 {
            return Err(KernelInteropError::RemoteGateIdTooLong);
        }
        if !id.is_ascii() {
            return Err(KernelInteropError::RemoteGateIdNotAscii);
        }
        Ok(Self(id))
    }
}

/// Canonical reference to a foreign canonical program/contract and its local counterpart.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VerifiedCounterpartRef {
    pub counterpart_id: CounterpartId,
    pub remote_chain_profile: ChainProfileId,
    pub remote_authority_domain: AuthorityDomain,
    pub remote_object_ref: String,
    pub remote_gate_id: RemoteGateId,
}

impl VerifiedCounterpartRef {
    pub fn validate(&self) -> Result<(), KernelInteropError> {
        if self.remote_object_ref.trim().is_empty() {
            return Err(KernelInteropError::EmptyRemoteObjectRef);
        }
        if self.remote_object_ref.len() > 192 {
            return Err(KernelInteropError::RemoteObjectRefTooLong);
        }
        if !self.remote_object_ref.is_ascii() {
            return Err(KernelInteropError::RemoteObjectRefNotAscii);
        }
        Ok(())
    }
}

/// Canonical cross-chain message key for replay protection and domain separation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CrossChainMessageId {
    pub source_profile: ChainProfileId,
    pub destination_profile: ChainProfileId,
    pub source_nonce: u64,
    pub routing_tag: [u8; 16],
}

impl CrossChainMessageId {
    pub fn validate(&self) -> Result<(), KernelInteropError> {
        if self.source_nonce == 0 {
            return Err(KernelInteropError::ZeroSourceNonce);
        }
        if self.routing_tag == [0u8; 16] {
            return Err(KernelInteropError::ZeroRoutingTag);
        }
        Ok(())
    }
}

/// Input surface for deterministic policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementContext {
    pub message_id: CrossChainMessageId,
    pub source_chain_compatibility: ChainCompatibilityClass,
    pub proof_type: ProofType,
    pub finality_class: FinalityClass,
    pub authority_domain: AuthorityDomain,
    pub direction: SettlementDirection,
    pub verified_counterpart: VerifiedCounterpartRef,
}

/// Kernel policy outcome emitted before execution dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementDecision {
    Accept,
    Defer,
    Reject,
}

/// Deterministic interaction status progression for paired remote/local coordination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RemoteInteractionStatus {
    Observed,
    Classified,
    CounterpartBound,
    AwaitingAcknowledgement,
    Acknowledged,
    Settled,
    Rejected,
    Expired,
}

/// Boundary contract for chain profile lookups.
pub trait ChainProfileRegistry {
    fn profile_exists(&self, profile_id: &ChainProfileId) -> bool;
}

/// Boundary contract for proof verification dispatch ownership.
pub trait ProofVerifierDispatcher {
    fn supports(&self, proof_type: ProofType) -> bool;
}

/// Boundary contract for policy decision ownership.
pub trait SettlementPolicyEngine {
    fn evaluate(&self, context: &SettlementContext) -> SettlementDecision;
}

/// Boundary contract for verified counterpart governance state.
pub trait VerifiedCounterpartRegistry {
    fn is_registered(&self, counterpart_id: &CounterpartId) -> bool;
}

/// Boundary contract for interaction progression and replay-sensitive status.
pub trait RemoteInteractionTracker {
    fn current_status(&self, message_id: &CrossChainMessageId) -> Option<RemoteInteractionStatus>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelInteropError {
    EmptyChainProfileId,
    ChainProfileIdTooLong,
    ChainProfileIdNotAscii,
    EmptyCounterpartId,
    CounterpartIdTooLong,
    CounterpartIdNotAscii,
    EmptyRemoteGateId,
    RemoteGateIdTooLong,
    RemoteGateIdNotAscii,
    EmptyRemoteObjectRef,
    RemoteObjectRefTooLong,
    RemoteObjectRefNotAscii,
    ZeroSourceNonce,
    ZeroRoutingTag,
}

impl fmt::Display for KernelInteropError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyChainProfileId => f.write_str("chain_profile_id must not be empty"),
            Self::ChainProfileIdTooLong => {
                f.write_str("chain_profile_id exceeds canonical maximum length")
            }
            Self::ChainProfileIdNotAscii => f.write_str("chain_profile_id must be ASCII"),
            Self::EmptyCounterpartId => f.write_str("counterpart_id must not be empty"),
            Self::CounterpartIdTooLong => {
                f.write_str("counterpart_id exceeds canonical maximum length")
            }
            Self::CounterpartIdNotAscii => f.write_str("counterpart_id must be ASCII"),
            Self::EmptyRemoteGateId => f.write_str("remote_gate_id must not be empty"),
            Self::RemoteGateIdTooLong => {
                f.write_str("remote_gate_id exceeds canonical maximum length")
            }
            Self::RemoteGateIdNotAscii => f.write_str("remote_gate_id must be ASCII"),
            Self::EmptyRemoteObjectRef => f.write_str("remote_object_ref must not be empty"),
            Self::RemoteObjectRefTooLong => {
                f.write_str("remote_object_ref exceeds canonical maximum length")
            }
            Self::RemoteObjectRefNotAscii => f.write_str("remote_object_ref must be ASCII"),
            Self::ZeroSourceNonce => f.write_str("source_nonce must not be zero"),
            Self::ZeroRoutingTag => f.write_str("routing_tag must not be all zeroes"),
        }
    }
}

impl std::error::Error for KernelInteropError {}

#[cfg(test)]
mod tests {
    use super::{
        AuthorityDomain, ChainCompatibilityClass, ChainProfileId, CounterpartId,
        CrossChainMessageId, KernelInteropError, RemoteGateId, SettlementContext,
        SettlementDirection, VerifiedCounterpartRef,
    };

    #[test]
    fn chain_profile_id_rejects_empty() {
        let error = ChainProfileId::new("  ").expect_err("empty profile id must fail");
        assert_eq!(error, KernelInteropError::EmptyChainProfileId);
    }

    #[test]
    fn cross_chain_message_id_requires_non_zero_routing_data() {
        let profile = ChainProfileId::new("evm-mainnet").expect("profile id should be valid");
        let id = CrossChainMessageId {
            source_profile: profile.clone(),
            destination_profile: profile,
            source_nonce: 0,
            routing_tag: [0u8; 16],
        };

        let error = id.validate().expect_err("invalid id must fail");
        assert_eq!(error, KernelInteropError::ZeroSourceNonce);
    }

    #[test]
    fn verified_counterpart_reference_rejects_empty_remote_object_ref() {
        let reference = VerifiedCounterpartRef {
            counterpart_id: CounterpartId::new("ethereum.usdc.proxy")
                .expect("counterpart id should be valid"),
            remote_chain_profile: ChainProfileId::new("evm-mainnet")
                .expect("chain profile id should be valid"),
            remote_authority_domain: AuthorityDomain::ContractGoverned,
            remote_object_ref: "   ".to_string(),
            remote_gate_id: RemoteGateId::new("gate.v1").expect("remote gate id should be valid"),
        };

        let error = reference
            .validate()
            .expect_err("empty remote object ref must fail");
        assert_eq!(error, KernelInteropError::EmptyRemoteObjectRef);
    }

    #[test]
    fn settlement_context_carries_counterpart_and_compatibility_boundaries() {
        let source = ChainProfileId::new("evm-mainnet").expect("source profile should be valid");
        let destination =
            ChainProfileId::new("aoxc-mainnet").expect("destination profile should be valid");
        let message_id = CrossChainMessageId {
            source_profile: source.clone(),
            destination_profile: destination,
            source_nonce: 9,
            routing_tag: [0xAB; 16],
        };

        let counterpart = VerifiedCounterpartRef {
            counterpart_id: CounterpartId::new("ethereum.staking_gate")
                .expect("counterpart id should be valid"),
            remote_chain_profile: source,
            remote_authority_domain: AuthorityDomain::ContractGoverned,
            remote_object_ref: "0xA0B86991C6218B36C1D19D4A2E9EB0CE3606EB48".to_string(),
            remote_gate_id: RemoteGateId::new("staking_gate.v2")
                .expect("remote gate id should be valid"),
        };

        let context = SettlementContext {
            message_id,
            source_chain_compatibility: ChainCompatibilityClass::EvmLike,
            proof_type: super::ProofType::MerkleInclusion,
            finality_class: super::FinalityClass::ProbabilisticConfirmations,
            authority_domain: AuthorityDomain::ContractGoverned,
            direction: SettlementDirection::Inbound,
            verified_counterpart: counterpart,
        };

        assert!(context.message_id.validate().is_ok());
        assert!(context.verified_counterpart.validate().is_ok());
    }
}
