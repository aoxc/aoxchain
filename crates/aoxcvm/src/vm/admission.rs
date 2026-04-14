//! Phase-1 kernel admission checks that bind transaction envelope + execution context.

use crate::auth::{registry::AuthProfileId, signer::SignerClass};
use crate::config::vm::{VmConstructionPlan, VmProfileError};
use crate::context::{deterministic::DeterminismLimits, execution::ExecutionContext};
use crate::tx::{envelope::TxEnvelope, validation::ValidationPolicy};
use aoxcontract::{ContractClass, RuntimeBindingDescriptor};
use aoxcore::genesis::{GenesisConfigError, KernelOperation, NodePolicy, NodeRole};
use std::collections::HashSet;

/// Admission errors produced before instruction execution begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    Context(crate::context::execution::ContextError),
    TxValidation(crate::tx::validation::ValidationError),
    ContextTxChainMismatch,
    ContextTxGasMismatch,
    TxGasExceedsKernelLimit,
    RestrictedAuthProfileRequired,
    RestrictedAuthProfileMismatch,
    RestrictedAuthProfileRegistryMismatch,
    GovernanceActivationRequired,
    GovernanceSignerClassRequired,
    TxKindForbiddenForClass,
    InvalidVmProfile(VmProfileError),
    VmReplayWindowCapabilityRequired,
    VmMlDsaCapabilityRequired,
    VmSlhDsaCapabilityRequired,
    VmHybridCapabilityRequired,
    KernelPolicy(GenesisConfigError),
}

/// Registry-backed auth context resolved before execution-time policy checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveAuthProfile {
    pub profile_id: AuthProfileId,
    pub profile_version: u16,
    pub profile_name: String,
    pub signer_class: SignerClass,
}

impl From<crate::context::execution::ContextError> for AdmissionError {
    fn from(value: crate::context::execution::ContextError) -> Self {
        Self::Context(value)
    }
}

impl From<crate::tx::validation::ValidationError> for AdmissionError {
    fn from(value: crate::tx::validation::ValidationError) -> Self {
        Self::TxValidation(value)
    }
}

impl From<GenesisConfigError> for AdmissionError {
    fn from(value: GenesisConfigError) -> Self {
        Self::KernelPolicy(value)
    }
}

/// Runtime kernel authorization context used to bridge VM admission with
/// node-role policy enforcement.
#[derive(Debug, Clone)]
pub struct KernelAdmissionContext<'a> {
    pub node_policy: &'a NodePolicy,
    pub role: NodeRole,
    pub operation: KernelOperation,
    pub epoch: u64,
    pub submitted_signers: &'a [String],
    pub eligible_signers: &'a HashSet<String>,
}

/// Auth and replay capabilities resolved during admission for a concrete VM profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmAdmissionCapabilities {
    pub replay_window_enforced: bool,
    pub ml_dsa_enabled: bool,
    pub slh_dsa_enabled: bool,
    pub hybrid_bundle_enabled: bool,
}

/// Binds immutable execution context with envelope-level admission constraints.
pub fn validate_phase1_admission(
    context: &ExecutionContext,
    tx: &TxEnvelope,
    limits: DeterminismLimits,
    max_payload_bytes: usize,
) -> Result<(), AdmissionError> {
    context.validate(limits)?;

    crate::tx::validation::validate(
        tx,
        ValidationPolicy {
            expected_chain_id: context.environment.chain_id,
            max_payload_bytes,
        },
    )?;

    if tx.chain_id != context.environment.chain_id {
        return Err(AdmissionError::ContextTxChainMismatch);
    }

    if tx.fee_budget.gas_limit > limits.max_gas_limit {
        return Err(AdmissionError::TxGasExceedsKernelLimit);
    }

    if context.tx.gas_limit != tx.fee_budget.gas_limit {
        return Err(AdmissionError::ContextTxGasMismatch);
    }

    Ok(())
}

/// Phase-2 admission checks that must be applied against a resolved runtime
/// binding before execution begins.
pub fn validate_phase2_admission(
    binding: &RuntimeBindingDescriptor,
    tx: &TxEnvelope,
    active_auth_profile: Option<&str>,
) -> Result<(), AdmissionError> {
    let policy = &binding.resolved_profile.policy_profile;

    if policy.governance_activation_required
        && !matches!(
            tx.kind,
            crate::tx::kind::TxKind::Governance | crate::tx::kind::TxKind::System
        )
    {
        return Err(AdmissionError::GovernanceActivationRequired);
    }

    if !tx_kind_allowed_for_class(
        tx.kind,
        &binding.resolved_profile.contract_class,
        policy.governance_activation_required,
    ) {
        return Err(AdmissionError::TxKindForbiddenForClass);
    }

    if let Some(required_profile) = policy.restricted_to_auth_profile.as_deref() {
        let active = active_auth_profile.ok_or(AdmissionError::RestrictedAuthProfileRequired)?;
        if normalize_auth_profile(active) != Some(required_profile) {
            return Err(AdmissionError::RestrictedAuthProfileMismatch);
        }
    }

    Ok(())
}

/// Phase-3 admission checks that wire registry-backed auth identity into
/// runtime admission and governance signer-class enforcement.
pub fn validate_phase3_admission(
    binding: &RuntimeBindingDescriptor,
    tx: &TxEnvelope,
    active_profile: Option<&ActiveAuthProfile>,
) -> Result<(), AdmissionError> {
    let policy = &binding.resolved_profile.policy_profile;
    let phase2_profile = policy
        .restricted_to_auth_profile
        .as_deref()
        .or(active_profile.map(|p| p.profile_name.as_str()));
    validate_phase2_admission(binding, tx, phase2_profile)?;

    if let Some(required_profile) = policy.restricted_to_auth_profile.as_deref() {
        let active = active_profile.ok_or(AdmissionError::RestrictedAuthProfileRequired)?;
        if active.profile_name != required_profile {
            return Err(AdmissionError::RestrictedAuthProfileRegistryMismatch);
        }
    }

    if matches!(
        tx.kind,
        crate::tx::kind::TxKind::Governance | crate::tx::kind::TxKind::System
    ) {
        let active = active_profile.ok_or(AdmissionError::GovernanceSignerClassRequired)?;
        if matches!(active.signer_class, SignerClass::Application) {
            return Err(AdmissionError::GovernanceSignerClassRequired);
        }
    }

    Ok(())
}

/// Phase-4 admission checks that wire VM tx flow into kernel-level node policy.
pub fn validate_phase4_kernel_admission(
    kernel: &KernelAdmissionContext<'_>,
) -> Result<(), AdmissionError> {
    kernel.node_policy.enforce_kernel_operation(
        kernel.role,
        kernel.operation,
        kernel.epoch,
        kernel.submitted_signers,
        kernel.eligible_signers,
    )?;

    Ok(())
}

/// Validates that resolved runtime capabilities satisfy the selected VM profile.
pub fn validate_vm_profile_admission(
    plan: VmConstructionPlan,
    capabilities: VmAdmissionCapabilities,
) -> Result<(), AdmissionError> {
    let validated = plan.validate().map_err(AdmissionError::InvalidVmProfile)?;

    if validated.enforce_replay_window && !capabilities.replay_window_enforced {
        return Err(AdmissionError::VmReplayWindowCapabilityRequired);
    }
    if validated.signature_policy.require_ml_dsa && !capabilities.ml_dsa_enabled {
        return Err(AdmissionError::VmMlDsaCapabilityRequired);
    }
    if validated.signature_policy.require_slh_dsa && !capabilities.slh_dsa_enabled {
        return Err(AdmissionError::VmSlhDsaCapabilityRequired);
    }
    if validated.signature_policy.require_hybrid_bundle && !capabilities.hybrid_bundle_enabled {
        return Err(AdmissionError::VmHybridCapabilityRequired);
    }
    Ok(())
}

fn normalize_auth_profile(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    let valid = !trimmed.is_empty()
        && trimmed == value
        && trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.'));
    valid.then_some(trimmed)
}

fn tx_kind_allowed_for_class(
    tx_kind: crate::tx::kind::TxKind,
    class: &ContractClass,
    governance_required: bool,
) -> bool {
    use crate::tx::kind::TxKind;
    match class {
        ContractClass::Application => matches!(tx_kind, TxKind::UserCall),
        ContractClass::Governed => matches!(tx_kind, TxKind::Governance | TxKind::System),
        ContractClass::System => matches!(tx_kind, TxKind::Governance | TxKind::System),
        ContractClass::Package => {
            matches!(
                tx_kind,
                TxKind::PackagePublish | TxKind::System | TxKind::Governance
            )
        }
        ContractClass::PolicyBound => {
            if governance_required {
                matches!(tx_kind, TxKind::Governance | TxKind::System)
            } else {
                matches!(
                    tx_kind,
                    TxKind::UserCall | TxKind::Governance | TxKind::System
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        context::{
            block::BlockContext, call::CallContext, deterministic::DeterminismLimits,
            environment::EnvironmentContext, execution::ExecutionContext, origin::OriginContext,
            tx::TxContext,
        },
        tx::{
            envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload,
            validation::ValidationError,
        },
        vm::admission::{
            ActiveAuthProfile, AdmissionError, KernelAdmissionContext, VmAdmissionCapabilities,
            validate_phase1_admission, validate_phase2_admission, validate_phase3_admission,
            validate_phase4_kernel_admission, validate_vm_profile_admission,
        },
    };
    use aoxcontract::{
        ContractClass, ContractId, ExecutionProfileRef, LaneBinding, RuntimeBindingDescriptor,
        VmTarget,
    };
    use aoxcore::genesis::{KernelOperation, NodePolicy, NodeRole};
    use std::collections::HashSet;

    fn sample_context(gas_limit: u64) -> ExecutionContext {
        ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 100, 1_700_000_000_000, [4_u8; 32]),
            TxContext::new([9_u8; 32], 0, gas_limit, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        )
    }

    fn sample_tx(gas_limit: u64) -> TxEnvelope {
        TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(gas_limit, 1),
            TxPayload::new(vec![1, 2, 3]),
        )
    }

    #[test]
    fn vm_profile_admission_requires_quantum_capabilities() {
        let caps = VmAdmissionCapabilities {
            replay_window_enforced: true,
            ml_dsa_enabled: true,
            slh_dsa_enabled: false,
            hybrid_bundle_enabled: true,
        };

        let err = validate_vm_profile_admission(
            crate::config::vm::VmConstructionPlan::quantum_resistant(),
            caps,
        )
        .expect_err("must reject missing slh-dsa");

        assert_eq!(err, AdmissionError::VmSlhDsaCapabilityRequired);
    }

    #[test]
    fn vm_profile_admission_accepts_advanced_profile_with_replay() {
        let caps = VmAdmissionCapabilities {
            replay_window_enforced: true,
            ..VmAdmissionCapabilities::default()
        };
        validate_vm_profile_admission(crate::config::vm::VmConstructionPlan::advanced(), caps)
            .expect("advanced profile accepted");
    }

    fn sample_binding() -> RuntimeBindingDescriptor {
        RuntimeBindingDescriptor {
            contract_id: ContractId("c_aox_phase2".to_string()),
            vm_target: VmTarget::Wasm,
            lane_binding: LaneBinding::Wasm,
            execution_profile: ExecutionProfileRef("phase2-policy-bound".to_string()),
            resolved_profile: aoxcontract::ExecutionProfile::phase2_default(&VmTarget::Wasm),
        }
    }

    fn sample_active_profile() -> ActiveAuthProfile {
        ActiveAuthProfile {
            profile_id: crate::auth::registry::AuthProfileId::new(7),
            profile_version: 3,
            profile_name: "ops-v1".to_string(),
            signer_class: crate::auth::signer::SignerClass::Governance,
        }
    }

    fn quorum_signers() -> Vec<String> {
        vec![
            "quorum-1".to_string(),
            "quorum-2".to_string(),
            "quorum-3".to_string(),
            "quorum-4".to_string(),
            "quorum-5".to_string(),
            "quorum-6".to_string(),
            "quorum-7".to_string(),
        ]
    }

    #[test]
    fn accepts_matching_context_and_tx() {
        let ctx = sample_context(500_000);
        let tx = sample_tx(500_000);

        assert_eq!(
            validate_phase1_admission(&ctx, &tx, DeterminismLimits::default(), 64 * 1024),
            Ok(())
        );
    }

    #[test]
    fn rejects_context_tx_gas_mismatch() {
        let ctx = sample_context(500_000);
        let tx = sample_tx(123_000);

        assert_eq!(
            validate_phase1_admission(&ctx, &tx, DeterminismLimits::default(), 64 * 1024),
            Err(AdmissionError::ContextTxGasMismatch)
        );
    }

    #[test]
    fn rejects_empty_payload_from_tx_validation() {
        let ctx = sample_context(500_000);
        let tx = TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(500_000, 1),
            TxPayload::new(vec![]),
        );

        assert_eq!(
            validate_phase1_admission(&ctx, &tx, DeterminismLimits::default(), 64 * 1024),
            Err(AdmissionError::TxValidation(ValidationError::EmptyPayload))
        );
    }

    #[test]
    fn phase2_rejects_missing_required_auth_profile() {
        let mut binding = sample_binding();
        binding
            .resolved_profile
            .policy_profile
            .restricted_to_auth_profile = Some("ops-v1".to_string());

        let err = validate_phase2_admission(&binding, &sample_tx(500_000), None).unwrap_err();
        assert_eq!(err, AdmissionError::RestrictedAuthProfileRequired);
    }

    #[test]
    fn phase2_rejects_governance_required_for_user_call() {
        let mut binding = sample_binding();
        binding.resolved_profile.contract_class = ContractClass::Governed;
        binding
            .resolved_profile
            .policy_profile
            .governance_activation_required = true;

        let err =
            validate_phase2_admission(&binding, &sample_tx(500_000), Some("ops-v1")).unwrap_err();
        assert_eq!(err, AdmissionError::GovernanceActivationRequired);
    }

    #[test]
    fn phase2_accepts_matching_auth_profile_and_governance_tx() {
        let mut binding = sample_binding();
        binding.resolved_profile.contract_class = ContractClass::PolicyBound;
        binding
            .resolved_profile
            .policy_profile
            .restricted_to_auth_profile = Some("ops-v1".to_string());
        binding
            .resolved_profile
            .policy_profile
            .governance_activation_required = true;

        let governance_tx = TxEnvelope::new(
            2626,
            1,
            TxKind::Governance,
            FeeBudget::new(500_000, 1),
            TxPayload::new(vec![1, 2, 3]),
        );

        assert_eq!(
            validate_phase2_admission(&binding, &governance_tx, Some("ops-v1")),
            Ok(())
        );
    }

    #[test]
    fn phase2_rejects_forbidden_tx_kind_for_system_class() {
        let mut binding = sample_binding();
        binding.resolved_profile.contract_class = ContractClass::System;

        let err = validate_phase2_admission(&binding, &sample_tx(500_000), None).unwrap_err();
        assert_eq!(err, AdmissionError::TxKindForbiddenForClass);
    }

    #[test]
    fn phase3_rejects_registry_profile_name_mismatch() {
        let mut binding = sample_binding();
        binding
            .resolved_profile
            .policy_profile
            .restricted_to_auth_profile = Some("ops-v1".to_string());

        let mut active = sample_active_profile();
        active.profile_name = "ops-v2".to_string();

        let err = validate_phase3_admission(&binding, &sample_tx(500_000), Some(&active))
            .expect_err("profile mismatch should fail");
        assert_eq!(err, AdmissionError::RestrictedAuthProfileRegistryMismatch);
    }

    #[test]
    fn phase3_rejects_governance_tx_from_application_signer() {
        let mut binding = sample_binding();
        binding.resolved_profile.contract_class = ContractClass::Governed;

        let governance_tx = TxEnvelope::new(
            2626,
            1,
            TxKind::Governance,
            FeeBudget::new(500_000, 1),
            TxPayload::new(vec![1, 2, 3]),
        );

        let mut active = sample_active_profile();
        active.signer_class = crate::auth::signer::SignerClass::Application;

        let err = validate_phase3_admission(&binding, &governance_tx, Some(&active))
            .expect_err("application signer should fail");
        assert_eq!(err, AdmissionError::GovernanceSignerClassRequired);
    }

    #[test]
    fn phase4_accepts_kernel_authorized_operation() {
        let policy = NodePolicy::for_network_class(aoxcore::genesis::NetworkClass::PublicMainnet);
        let signers = quorum_signers();
        let eligible: HashSet<String> = signers.iter().cloned().collect();

        let kernel = KernelAdmissionContext {
            node_policy: &policy,
            role: NodeRole::Quorum,
            operation: KernelOperation::QuorumVote,
            epoch: 0,
            submitted_signers: &signers,
            eligible_signers: &eligible,
        };

        assert_eq!(validate_phase4_kernel_admission(&kernel), Ok(()));
    }

    #[test]
    fn phase4_rejects_kernel_unauthorized_operation() {
        let policy = NodePolicy::for_network_class(aoxcore::genesis::NetworkClass::PublicMainnet);
        let signers = quorum_signers();
        let eligible: HashSet<String> = signers.iter().cloned().collect();

        let kernel = KernelAdmissionContext {
            node_policy: &policy,
            role: NodeRole::Seal,
            operation: KernelOperation::QuorumVote,
            epoch: 0,
            submitted_signers: &signers,
            eligible_signers: &eligible,
        };

        let err = validate_phase4_kernel_admission(&kernel).unwrap_err();
        assert!(matches!(err, AdmissionError::KernelPolicy(_)));
    }
}
