//! Phase-1 kernel admission checks that bind transaction envelope + execution context.

use crate::context::{deterministic::DeterminismLimits, execution::ExecutionContext};
use crate::tx::{envelope::TxEnvelope, validation::ValidationPolicy};
use aoxcontract::{ContractClass, RuntimeBindingDescriptor};

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
    GovernanceActivationRequired,
    TxKindForbiddenForClass,
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
    if !tx_kind_allowed_for_class(
        tx.kind,
        &binding.resolved_profile.contract_class,
        binding
            .resolved_profile
            .policy_profile
            .governance_activation_required,
    ) {
        return Err(AdmissionError::TxKindForbiddenForClass);
    }

    let policy = &binding.resolved_profile.policy_profile;

    if policy.governance_activation_required
        && !matches!(
            tx.kind,
            crate::tx::kind::TxKind::Governance | crate::tx::kind::TxKind::System
        )
    {
        return Err(AdmissionError::GovernanceActivationRequired);
    }

    if let Some(required_profile) = policy.restricted_to_auth_profile.as_deref() {
        let active = active_auth_profile.ok_or(AdmissionError::RestrictedAuthProfileRequired)?;
        if normalize_auth_profile(active) != Some(required_profile) {
            return Err(AdmissionError::RestrictedAuthProfileMismatch);
        }
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
        vm::admission::validate_phase2_admission,
        vm::admission::{AdmissionError, validate_phase1_admission},
    };
    use aoxcontract::{
        ContractClass, ContractId, ExecutionProfileRef, LaneBinding, RuntimeBindingDescriptor,
        VmTarget,
    };

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

    fn sample_binding() -> RuntimeBindingDescriptor {
        RuntimeBindingDescriptor {
            contract_id: ContractId("c_aox_phase2".to_string()),
            vm_target: VmTarget::Wasm,
            lane_binding: LaneBinding::Wasm,
            execution_profile: ExecutionProfileRef("phase2-policy-bound".to_string()),
            resolved_profile: aoxcontract::ExecutionProfile::phase2_default(&VmTarget::Wasm),
        }
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
}
