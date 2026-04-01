// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::BTreeSet;

use crate::error::AovmError;
use crate::language::LanguageKind;
use crate::vm_kind::VmKind;

/// Canonical relay envelope evaluated by language adapters before execution
/// admission or settlement progression.
///
/// This structure is intentionally minimal and transport-agnostic. It carries
/// only the fields required for deterministic pre-execution conformance checks.
/// Higher-level proof interpretation, routing semantics, and settlement policy
/// remain external concerns and must not be implicitly inferred here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayEnvelope {
    /// Canonical message identifier used for replay-protection and batch uniqueness checks.
    pub message_id: [u8; 32],

    /// Source chain identifier from which the relay intent originated.
    pub source_chain_id: u64,

    /// Target chain identifier for which the relay intent is destined.
    pub target_chain_id: u64,

    /// Monotonic relay nonce carried by the producing system.
    pub nonce: u64,

    /// Canonical payload subject to downstream execution or settlement handling.
    pub payload: Vec<u8>,

    /// Opaque finality evidence required for relay admission.
    pub finality_proof: Vec<u8>,
}

impl RelayEnvelope {
    /// Performs structural validation that is independent from any
    /// language-specific or VM-specific semantics.
    ///
    /// # Validation Guarantees
    /// - source and target chains must be distinct;
    /// - relay payload must be present;
    /// - finality evidence must be present.
    ///
    /// # Security Rationale
    /// These checks establish a minimal fail-closed baseline before adapter-level
    /// policy is evaluated. The function must remain deterministic and free from
    /// contextual side effects.
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

/// Defines the language-first admission contract applied to canonical relay data.
///
/// The trait intentionally exposes a constrained surface:
/// - language family classification,
/// - VM lane classification,
/// - replay domain identity,
/// - deterministic relay validation.
///
/// Implementations must remain pure with respect to identical inputs.
pub trait LanguageAdapter {
    /// Returns the language family represented by this adapter.
    fn language_kind(&self) -> LanguageKind;

    /// Returns the VM lane associated with this adapter.
    fn vm_kind(&self) -> VmKind;

    /// Returns the canonical replay domain used to scope relay uniqueness and
    /// semantic separation for this adapter profile.
    fn replay_domain(&self) -> &'static str;

    /// Validates the supplied relay envelope under the adapter’s deterministic
    /// admission rules.
    fn validate_relay(&self, envelope: &RelayEnvelope) -> Result<(), AovmError>;
}

/// Immutable adapter profile bound to a specific VM lane and replay domain.
///
/// This adapter is intentionally lightweight and copyable so that registry-level
/// resolution remains side-effect free and inexpensive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmLanguageAdapter {
    vm_kind: VmKind,
    replay_domain: &'static str,
}

impl VmLanguageAdapter {
    /// Constructs a new immutable VM-language adapter profile.
    ///
    /// # Parameters
    /// - `vm_kind`: The VM lane represented by the adapter.
    /// - `replay_domain`: Canonical replay-domain namespace for the adapter.
    #[must_use]
    pub const fn new(vm_kind: VmKind, replay_domain: &'static str) -> Self {
        Self {
            vm_kind,
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

    fn replay_domain(&self) -> &'static str {
        self.replay_domain
    }

    fn validate_relay(&self, envelope: &RelayEnvelope) -> Result<(), AovmError> {
        envelope.validate_basic()?;

        if self.replay_domain().is_empty() {
            return Err(AovmError::InvalidTransaction("replay domain is empty"));
        }

        Ok(())
    }
}

/// Returns the canonical default adapter profile for the supplied VM lane.
///
/// This function centralizes lane-to-adapter mapping so that kernel-facing
/// consumers do not duplicate replay-domain selection logic.
///
/// # Design Constraints
/// - mapping must be deterministic;
/// - mapping must not depend on mutable global state;
/// - replay-domain strings must remain stable unless an explicit migration is introduced.
#[must_use]
pub fn default_adapter_for_vm(vm_kind: VmKind) -> VmLanguageAdapter {
    let replay_domain = match vm_kind {
        VmKind::Evm => "aovm/relay/evm",
        VmKind::SuiMove => "aovm/relay/move",
        VmKind::Wasm => "aovm/relay/wasm",
        VmKind::Cardano => "aovm/relay/cardano",
    };

    VmLanguageAdapter::new(vm_kind, replay_domain)
}

/// Executes deterministic conformance checks over a relay batch using the
/// supplied adapter profile.
///
/// # Validation Scope
/// - duplicate message identifiers are rejected;
/// - each relay envelope is validated by the adapter;
/// - a second validation pass is executed as a determinism sanity check.
///
/// # Security Rationale
/// The second pass is intentionally simple. Its purpose is not to model full
/// replay execution, but to ensure that repeated validation of identical inputs
/// does not produce divergent outcomes under the same adapter profile.
pub fn conformance_check<A: LanguageAdapter>(
    adapter: &A,
    envelopes: &[RelayEnvelope],
) -> Result<(), AovmError> {
    let mut seen_message_ids = BTreeSet::new();

    for envelope in envelopes {
        if !seen_message_ids.insert(envelope.message_id) {
            return Err(AovmError::InvalidTransaction("duplicate relay message id"));
        }

        adapter.validate_relay(envelope)?;
    }

    for envelope in envelopes {
        adapter.validate_relay(envelope)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LanguageAdapter, RelayEnvelope, VmLanguageAdapter, conformance_check,
        default_adapter_for_vm,
    };
    use crate::language::LanguageKind;
    use crate::vm_kind::VmKind;

    fn envelope(id: u8) -> RelayEnvelope {
        RelayEnvelope {
            message_id: [id; 32],
            source_chain_id: 1,
            target_chain_id: 2,
            nonce: u64::from(id),
            payload: b"relay-intent".to_vec(),
            finality_proof: b"proof".to_vec(),
        }
    }

    #[test]
    fn adapter_maps_vm_to_language_family() {
        let adapter = VmLanguageAdapter::new(VmKind::SuiMove, "move/domain");

        assert_eq!(adapter.vm_kind(), VmKind::SuiMove);
        assert_eq!(adapter.language_kind(), LanguageKind::Move);
    }

    #[test]
    fn default_adapter_mapping_uses_expected_replay_domains() {
        let cases = [
            (VmKind::Evm, "aovm/relay/evm"),
            (VmKind::SuiMove, "aovm/relay/move"),
            (VmKind::Wasm, "aovm/relay/wasm"),
            (VmKind::Cardano, "aovm/relay/cardano"),
        ];

        for (vm_kind, expected_domain) in cases {
            let adapter = default_adapter_for_vm(vm_kind);
            assert_eq!(adapter.vm_kind(), vm_kind);
            assert_eq!(adapter.replay_domain(), expected_domain);
        }
    }

    #[test]
    fn relay_envelope_rejects_identical_source_and_target_chain() {
        let envelope = RelayEnvelope {
            message_id: [9; 32],
            source_chain_id: 55,
            target_chain_id: 55,
            nonce: 1,
            payload: b"payload".to_vec(),
            finality_proof: b"proof".to_vec(),
        };

        let err = envelope
            .validate_basic()
            .expect_err("source and target chain equality must be rejected");

        assert_eq!(
            err.to_string(),
            "invalid transaction: source and target chain must differ"
        );
    }

    #[test]
    fn relay_envelope_rejects_empty_payload() {
        let envelope = RelayEnvelope {
            message_id: [10; 32],
            source_chain_id: 1,
            target_chain_id: 2,
            nonce: 1,
            payload: Vec::new(),
            finality_proof: b"proof".to_vec(),
        };

        let err = envelope
            .validate_basic()
            .expect_err("empty payload must be rejected");

        assert_eq!(
            err.to_string(),
            "invalid transaction: relay payload is empty"
        );
    }

    #[test]
    fn relay_envelope_rejects_empty_finality_proof() {
        let envelope = RelayEnvelope {
            message_id: [11; 32],
            source_chain_id: 1,
            target_chain_id: 2,
            nonce: 1,
            payload: b"payload".to_vec(),
            finality_proof: Vec::new(),
        };

        let err = envelope
            .validate_basic()
            .expect_err("empty finality proof must be rejected");

        assert_eq!(
            err.to_string(),
            "invalid transaction: finality proof is empty"
        );
    }

    #[test]
    fn conformance_rejects_duplicate_message_ids() {
        let adapter = VmLanguageAdapter::new(VmKind::Evm, "evm/domain");
        let envelopes = vec![envelope(7), envelope(7)];

        let err = conformance_check(&adapter, &envelopes)
            .expect_err("duplicate relay message identifiers must be rejected");

        assert_eq!(
            err.to_string(),
            "invalid transaction: duplicate relay message id"
        );
    }

    #[test]
    fn conformance_accepts_unique_proved_envelopes() {
        let adapter = VmLanguageAdapter::new(VmKind::Wasm, "wasm/domain");
        let envelopes = vec![envelope(1), envelope(2), envelope(3)];

        conformance_check(&adapter, &envelopes)
            .expect("unique proved envelopes must satisfy conformance");
    }

    #[test]
    fn default_adapter_profiles_expose_non_empty_replay_domains_for_all_current_lanes() {
        for vm_kind in [VmKind::Evm, VmKind::SuiMove, VmKind::Wasm, VmKind::Cardano] {
            let adapter = default_adapter_for_vm(vm_kind);
            assert!(
                !adapter.replay_domain().is_empty(),
                "default replay domain must not be empty for {:?}",
                vm_kind
            );
        }
    }
}
