// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::error::AovmError;
use crate::language_adapter::{
    conformance_check, default_adapter_for_vm, RelayEnvelope, VmLanguageAdapter,
};
use crate::vm_kind::VmKind;

/// Provides a minimal registry facade for resolving the canonical default
/// language adapter associated with a given VM lane.
///
/// This abstraction deliberately remains narrow in scope:
/// - adapter selection is delegated to the language-adapter layer;
/// - batch validation is executed through deterministic conformance checks;
/// - no mutable registry state is introduced at this stage.
///
/// The design intent is to preserve a stable kernel-facing surface while
/// keeping adapter resolution logic centralized and auditable.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdapterRegistry;

impl AdapterRegistry {
    /// Resolves the canonical default adapter for the supplied VM lane.
    ///
    /// # Parameters
    /// - `vm_kind`: The execution lane for which the default adapter must be resolved.
    ///
    /// # Returns
    /// The deterministic default [`VmLanguageAdapter`] bound to the specified VM kind.
    ///
    /// # Design Note
    /// This function is intentionally side-effect free. Registry resolution must
    /// remain deterministic and must not depend on runtime-global mutable state.
    #[must_use]
    pub fn adapter_for_vm(&self, vm_kind: VmKind) -> VmLanguageAdapter {
        default_adapter_for_vm(vm_kind)
    }

    /// Validates a batch of relay envelopes against the default adapter profile
    /// of the specified VM lane.
    ///
    /// # Parameters
    /// - `vm_kind`: The execution lane whose canonical adapter profile will be used.
    /// - `envelopes`: The relay envelopes to be validated.
    ///
    /// # Returns
    /// - `Ok(())` when the batch satisfies deterministic conformance requirements.
    /// - `Err(AovmError)` when the batch violates relay-admission invariants.
    ///
    /// # Security Rationale
    /// Validation is executed only after canonical adapter resolution. This avoids
    /// ambiguous policy selection and ensures that batch admission semantics remain
    /// stable for identical inputs.
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
            nonce: u64::from(id),
            payload: b"registry-relay".to_vec(),
            finality_proof: b"proof".to_vec(),
        }
    }

    #[test]
    fn registry_returns_non_empty_replay_domains_for_all_supported_vm_lanes() {
        let registry = AdapterRegistry;

        for vm_kind in [VmKind::Evm, VmKind::SuiMove, VmKind::Wasm, VmKind::Cardano] {
            let adapter = registry.adapter_for_vm(vm_kind);
            assert!(
                !adapter.replay_domain().is_empty(),
                "default adapter replay domain must not be empty for {:?}",
                vm_kind
            );
        }
    }

    #[test]
    fn registry_validate_batch_rejects_duplicate_message_identifiers() {
        let registry = AdapterRegistry;
        let envelopes = vec![envelope(4), envelope(4)];

        let err = registry
            .validate_batch(VmKind::Evm, &envelopes)
            .expect_err("duplicate relay message identifiers must be rejected");

        assert_eq!(
            err.to_string(),
            "invalid transaction: duplicate relay message id"
        );
    }

    #[test]
    fn registry_validate_batch_accepts_conformant_batches() {
        let registry = AdapterRegistry;
        let envelopes = vec![envelope(1), envelope(2), envelope(3)];

        registry
            .validate_batch(VmKind::Wasm, &envelopes)
            .expect("conformant relay batches must be accepted");
    }
}
