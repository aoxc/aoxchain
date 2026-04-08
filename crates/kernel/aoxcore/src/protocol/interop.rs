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
    pub proof_type: ProofType,
    pub finality_class: FinalityClass,
    pub authority_domain: AuthorityDomain,
    pub direction: SettlementDirection,
}

/// Kernel policy outcome emitted before execution dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementDecision {
    Accept,
    Defer,
    Reject,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelInteropError {
    EmptyChainProfileId,
    ChainProfileIdTooLong,
    ChainProfileIdNotAscii,
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
            Self::ZeroSourceNonce => f.write_str("source_nonce must not be zero"),
            Self::ZeroRoutingTag => f.write_str("routing_tag must not be all zeroes"),
        }
    }
}

impl std::error::Error for KernelInteropError {}

#[cfg(test)]
mod tests {
    use super::{ChainProfileId, CrossChainMessageId, KernelInteropError};

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
}
