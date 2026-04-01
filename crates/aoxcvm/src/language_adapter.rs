// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeSet;

use crate::error::AovmError;
use crate::language::LanguageKind;
use crate::vm_kind::VmKind;

/// Finality proof category accepted by a relay adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalityProofKind {
    LightClientInclusion,
    ValidityProof,
    CheckpointAttestation,
}

/// Deterministic replay domain identifier used by kernel admission checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayDomainTag(pub Vec<u8>);

impl ReplayDomainTag {
    pub fn new(language: LanguageKind, vm: VmKind, domain: &str) -> Result<Self, AovmError> {
        if domain.is_empty() {
            return Err(AovmError::InvalidTransaction("replay domain is empty"));
        }
        let tag = format!("aoxclang:{}:{}:{domain}", language.as_id(), vm.as_str()).into_bytes();
        Ok(Self(tag))
    }
}

/// Canonical relay envelope validated by language adapters before settlement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayEnvelope {
    pub message_id: [u8; 32],
    pub source_chain_id: u64,
    pub target_chain_id: u64,
    pub nonce: u64,
    pub payload: Vec<u8>,
    pub finality_proof: Vec<u8>,
}

impl RelayEnvelope {
    pub fn validate_basic(&self) -> Result<(), AovmError> {
        if self.source_chain_id == self.target_chain_id {
            return Err(AovmError::InvalidTransaction(
                "source and target chain must differ",
            ));
        }
        if self.payload.is_empty() {
            return Err(AovmError::InvalidTransaction("relay payload is empty"));
        }
        if self.finality_proof.is_empty() {
            return Err(AovmError::InvalidTransaction("finality proof is empty"));
        }
        Ok(())
    }
}

/// Adapter contract for language-first relay admission checks.
pub trait LanguageAdapter {
    fn language_kind(&self) -> LanguageKind;
    fn vm_kind(&self) -> VmKind;
    fn proof_kind(&self) -> FinalityProofKind;
    fn replay_domain(&self) -> &'static str;
    fn replay_tag(&self) -> Result<ReplayDomainTag, AovmError>;
    fn validate_relay(&self, envelope: &RelayEnvelope) -> Result<(), AovmError>;
}

/// Simple adapter implementation used for kernel conformance validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmLanguageAdapter {
    vm_kind: VmKind,
    proof_kind: FinalityProofKind,
    replay_domain: &'static str,
}

impl VmLanguageAdapter {
    pub const fn new(
        vm_kind: VmKind,
        proof_kind: FinalityProofKind,
        replay_domain: &'static str,
    ) -> Self {
        Self {
            vm_kind,
            proof_kind,
            replay_domain,
        }
    }
}

impl LanguageAdapter for VmLanguageAdapter {
    fn language_kind(&self) -> LanguageKind {
        self.vm_kind.language_kind()
    }

    fn vm_kind(&self) -> VmKind {
        self.vm_kind
    }

    fn proof_kind(&self) -> FinalityProofKind {
        self.proof_kind
    }

    fn replay_domain(&self) -> &'static str {
        self.replay_domain
    }

    fn replay_tag(&self) -> Result<ReplayDomainTag, AovmError> {
        ReplayDomainTag::new(self.language_kind(), self.vm_kind(), self.replay_domain())
    }

    fn validate_relay(&self, envelope: &RelayEnvelope) -> Result<(), AovmError> {
        envelope.validate_basic()?;
        let _tag = self.replay_tag()?;
        Ok(())
    }
}

/// Returns the default adapter profile for a VM lane.
pub const fn default_adapter_for_vm(vm_kind: VmKind) -> VmLanguageAdapter {
    match vm_kind {
        VmKind::Evm => VmLanguageAdapter::new(
            VmKind::Evm,
            FinalityProofKind::LightClientInclusion,
            "evm/default",
        ),
        VmKind::SuiMove => VmLanguageAdapter::new(
            VmKind::SuiMove,
            FinalityProofKind::CheckpointAttestation,
            "move/default",
        ),
        VmKind::Wasm => VmLanguageAdapter::new(
            VmKind::Wasm,
            FinalityProofKind::LightClientInclusion,
            "wasm/default",
        ),
        VmKind::Cardano => VmLanguageAdapter::new(
            VmKind::Cardano,
            FinalityProofKind::CheckpointAttestation,
            "cardano/default",
        ),
    }
}

/// Runs deterministic conformance checks for an adapter against relay envelopes.
pub fn conformance_check<A: LanguageAdapter>(
    adapter: &A,
    envelopes: &[RelayEnvelope],
) -> Result<(), AovmError> {
    let mut seen = BTreeSet::new();
    for envelope in envelopes {
        if !seen.insert(envelope.message_id) {
            return Err(AovmError::InvalidTransaction("duplicate relay message id"));
        }
        adapter.validate_relay(envelope)?;
    }

    // Determinism sanity: a second pass over identical envelopes must match.
    for envelope in envelopes {
        adapter.validate_relay(envelope)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        FinalityProofKind, LanguageAdapter, RelayEnvelope, VmLanguageAdapter, conformance_check,
        default_adapter_for_vm,
    };
    use crate::language::LanguageKind;
    use crate::vm_kind::VmKind;

    fn envelope(id: u8) -> RelayEnvelope {
        RelayEnvelope {
            message_id: [id; 32],
            source_chain_id: 1,
            target_chain_id: 2,
            nonce: id as u64,
            payload: b"relay-intent".to_vec(),
            finality_proof: b"proof".to_vec(),
        }
    }

    #[test]
    fn adapter_maps_vm_to_language_family() {
        let adapter = VmLanguageAdapter::new(
            VmKind::SuiMove,
            FinalityProofKind::CheckpointAttestation,
            "move/domain",
        );
        assert_eq!(adapter.vm_kind(), VmKind::SuiMove);
        assert_eq!(adapter.language_kind(), LanguageKind::Move);
        assert_eq!(adapter.proof_kind(), FinalityProofKind::CheckpointAttestation);
    }

    #[test]
    fn conformance_rejects_duplicate_message_ids() {
        let adapter = VmLanguageAdapter::new(
            VmKind::Evm,
            FinalityProofKind::LightClientInclusion,
            "evm/domain",
        );
        let envelopes = vec![envelope(7), envelope(7)];
        let err = conformance_check(&adapter, &envelopes).expect_err("must reject duplicates");
        assert_eq!(
            err.to_string(),
            "invalid transaction: duplicate relay message id"
        );
    }

    #[test]
    fn conformance_accepts_unique_proved_envelopes() {
        let adapter = VmLanguageAdapter::new(
            VmKind::Wasm,
            FinalityProofKind::LightClientInclusion,
            "wasm/domain",
        );
        let envelopes = vec![envelope(1), envelope(2), envelope(3)];
        conformance_check(&adapter, &envelopes).expect("envelopes should conform");
    }

    #[test]
    fn default_adapters_have_stable_replay_tags() {
        for vm in [VmKind::Evm, VmKind::SuiMove, VmKind::Wasm, VmKind::Cardano] {
            let adapter = default_adapter_for_vm(vm);
            let tag = adapter
                .replay_tag()
                .expect("default adapters must emit replay tags");
            assert!(!tag.0.is_empty());
        }
    }
}
