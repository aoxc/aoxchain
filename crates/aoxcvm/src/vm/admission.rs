//! Phase-1 kernel admission checks that bind transaction envelope + execution context.

use crate::context::{deterministic::DeterminismLimits, execution::ExecutionContext};
use crate::tx::{envelope::TxEnvelope, validation::ValidationPolicy};

/// Admission errors produced before instruction execution begins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    Context(crate::context::execution::ContextError),
    TxValidation(crate::tx::validation::ValidationError),
    ContextTxChainMismatch,
    ContextTxGasMismatch,
    TxGasExceedsKernelLimit,
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
        vm::admission::{AdmissionError, validate_phase1_admission},
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
}
