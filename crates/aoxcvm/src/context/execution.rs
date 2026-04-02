//! Immutable execution context for phase-1 kernel admission and runtime binding.

use crate::context::{
    block::BlockContext, call::CallContext, deterministic::DeterminismLimits,
    environment::EnvironmentContext, origin::OriginContext, tx::TxContext,
};

/// Immutable canonical context provided to a single kernel execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionContext {
    pub environment: EnvironmentContext,
    pub block: BlockContext,
    pub tx: TxContext,
    pub call: CallContext,
    pub origin: OriginContext,
}

/// Context admission failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextError {
    ZeroChainId,
    ZeroNetworkId,
    ZeroGasLimit,
    GasLimitExceedsDeterminism,
    DepthExceedsDeterminism,
    SpecVersionTooOld,
}

impl ExecutionContext {
    pub const fn new(
        environment: EnvironmentContext,
        block: BlockContext,
        tx: TxContext,
        call: CallContext,
        origin: OriginContext,
    ) -> Self {
        Self {
            environment,
            block,
            tx,
            call,
            origin,
        }
    }

    /// Validates minimal phase-1 context admission checks.
    pub fn validate(&self, limits: DeterminismLimits) -> Result<(), ContextError> {
        if self.environment.chain_id == 0 {
            return Err(ContextError::ZeroChainId);
        }
        if self.environment.network_id == 0 {
            return Err(ContextError::ZeroNetworkId);
        }
        if self.tx.gas_limit == 0 {
            return Err(ContextError::ZeroGasLimit);
        }
        if self.tx.gas_limit > limits.max_gas_limit {
            return Err(ContextError::GasLimitExceedsDeterminism);
        }
        if self.call.depth > limits.max_call_depth {
            return Err(ContextError::DepthExceedsDeterminism);
        }
        if self.tx.spec_version < limits.min_spec_version {
            return Err(ContextError::SpecVersionTooOld);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::context::{
        block::BlockContext, call::CallContext, deterministic::DeterminismLimits,
        environment::EnvironmentContext, execution::ContextError, origin::OriginContext,
        tx::TxContext,
    };

    use super::ExecutionContext;

    fn sample_context() -> ExecutionContext {
        ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(10, 250, 1_700_000_000_000, [7_u8; 32]),
            TxContext::new([3_u8; 32], 0, 100_000, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        )
    }

    #[test]
    fn accepts_valid_context() {
        let context = sample_context();
        assert_eq!(context.validate(DeterminismLimits::default()), Ok(()));
    }

    #[test]
    fn rejects_invalid_chain_id() {
        let mut context = sample_context();
        context.environment.chain_id = 0;
        assert_eq!(
            context.validate(DeterminismLimits::default()),
            Err(ContextError::ZeroChainId)
        );
    }

    #[test]
    fn rejects_depth_over_limit() {
        let mut context = sample_context();
        context.call.depth = 99;
        assert_eq!(
            context.validate(DeterminismLimits::default()),
            Err(ContextError::DepthExceedsDeterminism)
        );
    }
}
