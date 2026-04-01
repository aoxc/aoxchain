// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::AovmError;
use crate::language_adapter::{
    RelayEnvelope, VmLanguageAdapter, conformance_check, default_adapter_for_vm,
};
use crate::vm_kind::VmKind;

/// Registry facade that exposes kernel-default adapters for registered VM lanes.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterRegistry;

impl AdapterRegistry {
    /// Returns the default adapter for a VM lane.
    pub fn adapter_for_vm(&self, vm_kind: VmKind) -> VmLanguageAdapter {
        default_adapter_for_vm(vm_kind)
    }

    /// Runs relay conformance checks using the lane default adapter profile.
    pub fn validate_batch(
        &self,
        vm_kind: VmKind,
        envelopes: &[RelayEnvelope],
    ) -> Result<(), AovmError> {
        let adapter = self.adapter_for_vm(vm_kind);
        conformance_check(&adapter, envelopes)
    }
}

#[cfg(test)]
mod tests {
    use super::AdapterRegistry;
    use crate::language_adapter::{LanguageAdapter, RelayEnvelope};
    use crate::vm_kind::VmKind;

    fn envelope(id: u8) -> RelayEnvelope {
        RelayEnvelope {
            message_id: [id; 32],
            source_chain_id: 10,
            target_chain_id: 20,
            nonce: id as u64,
            payload: b"registry-relay".to_vec(),
            finality_proof: b"proof".to_vec(),
        }
    }

    #[test]
    fn registry_returns_non_empty_replay_domains_for_all_current_lanes() {
        let registry = AdapterRegistry;
        for vm in [VmKind::Evm, VmKind::SuiMove, VmKind::Wasm, VmKind::Cardano] {
            let adapter = registry.adapter_for_vm(vm);
            assert!(!adapter.replay_domain().is_empty());
        }
    }

    #[test]
    fn registry_validate_batch_detects_duplicates() {
        let registry = AdapterRegistry;
        let envelopes = vec![envelope(4), envelope(4)];
        let err = registry
            .validate_batch(VmKind::Evm, &envelopes)
            .expect_err("duplicate IDs must fail");
        assert_eq!(
            err.to_string(),
            "invalid transaction: duplicate relay message id"
        );
    }

    #[test]
    fn registry_validate_batch_accepts_valid_batch() {
        let registry = AdapterRegistry;
        let envelopes = vec![envelope(1), envelope(2), envelope(3)];
        registry
            .validate_batch(VmKind::Wasm, &envelopes)
            .expect("valid batch must pass");
    }
}
