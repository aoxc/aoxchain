//! AOXC-VMachine-QX1 phase-1 kernel orchestration.

use crate::context::{
    block::BlockContext,
    call::CallContext,
    deterministic::DeterminismLimits,
    environment::EnvironmentContext,
    execution::{ContextError, ExecutionContext},
    origin::OriginContext,
    tx::TxContext,
};
use crate::receipts::proof::ReceiptProof;
use crate::state::JournaledState;
use crate::verifier::determinism::{DeterminismError, DeterminismVerifier};
use crate::vm::machine::{ExecutionResult, Program};

/// Configuration for a phase-1 kernel run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelConfig {
    pub gas_limit: u64,
    pub max_memory: usize,
    pub max_call_depth: u16,
    pub min_spec_version: u32,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            gas_limit: 1_000_000,
            max_memory: 1024 * 1024,
            max_call_depth: 64,
            min_spec_version: 1,
        }
    }
}

/// Minimal full phase-1 output surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelOutput {
    pub context: ExecutionContext,
    pub result: ExecutionResult,
    pub receipt_proof: ReceiptProof,
}

impl KernelOutput {
    /// Returns final deterministic state snapshot source.
    pub fn final_state(&self) -> &JournaledState {
        &self.result.final_state
    }
}

/// Phase-1 kernel execution failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    Context(ContextError),
    Determinism(DeterminismError),
}

impl From<ContextError> for KernelError {
    fn from(value: ContextError) -> Self {
        Self::Context(value)
    }
}

impl From<DeterminismError> for KernelError {
    fn from(value: DeterminismError) -> Self {
        Self::Determinism(value)
    }
}

/// AOXC-VMachine-QX1 phase-1 kernel entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AOXCVMachineQX1 {
    config: KernelConfig,
}

impl AOXCVMachineQX1 {
    /// Creates a kernel with deterministic execution limits.
    pub fn new(config: KernelConfig) -> Self {
        Self { config }
    }

    /// Executes and verifies a program with deterministic replay using a default context.
    pub fn execute_phase1(&self, program: Program) -> Result<KernelOutput, KernelError> {
        self.execute_phase1_with_context(program, self.default_context())
    }

    /// Executes and verifies a program using caller-provided immutable execution context.
    pub fn execute_phase1_with_context(
        &self,
        program: Program,
        context: ExecutionContext,
    ) -> Result<KernelOutput, KernelError> {
        context.validate(DeterminismLimits {
            max_call_depth: self.config.max_call_depth,
            max_gas_limit: self.config.gas_limit,
            min_spec_version: self.config.min_spec_version,
        })?;

        let verifier = DeterminismVerifier {
            gas_limit: context.tx.gas_limit,
            max_memory: self.config.max_memory,
        };

        let result = verifier.verify(program)?;
        let receipt_proof = ReceiptProof::new(&result.receipt, 2);
        Ok(KernelOutput {
            context,
            result,
            receipt_proof,
        })
    }

    fn default_context(&self) -> ExecutionContext {
        ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(0, 0, 0, [0_u8; 32]),
            TxContext::new([0_u8; 32], 0, self.config.gas_limit, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([0_u8; 32], [0_u8; 32], [0_u8; 32], 0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AOXCVMachineQX1, KernelConfig, KernelError};
    use crate::context::{
        block::BlockContext, call::CallContext, environment::EnvironmentContext,
        execution::ExecutionContext, origin::OriginContext, tx::TxContext,
    };
    use crate::vm::machine::{Instruction, Program};

    #[test]
    fn phase1_kernel_executes_full_minimal_surface() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 1_000,
            max_memory: 1024,
            max_call_depth: 64,
            min_spec_version: 1,
        });

        let program = Program {
            code: vec![
                Instruction::Push(11),
                Instruction::StoreMem { offset: 0 },
                Instruction::Push(1),
                Instruction::LoadMem { offset: 0 },
                Instruction::SStore,
                Instruction::Push(1),
                Instruction::SLoad,
                Instruction::LogTop,
                Instruction::Halt,
            ],
        };

        let output = kernel.execute_phase1(program).expect("phase1 success");
        assert_eq!(output.result.stack, vec![11]);
        assert!(output.receipt_proof.verify_receipt(&output.result.receipt));
        assert_eq!(
            output.final_state().get(&1_u64.to_le_bytes()),
            Some(&11_u64.to_le_bytes()[..])
        );
    }

    #[test]
    fn rejects_context_depth_over_limit() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 1000,
            max_memory: 1024,
            max_call_depth: 4,
            min_spec_version: 1,
        });

        let context = ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 1, 1, [9_u8; 32]),
            TxContext::new([7_u8; 32], 0, 500, false, 1, 0),
            CallContext::new(8),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        );

        let err = kernel
            .execute_phase1_with_context(
                Program {
                    code: vec![Instruction::Halt],
                },
                context,
            )
            .expect_err("must fail");

        assert!(matches!(
            err,
            KernelError::Context(crate::context::execution::ContextError::DepthExceedsDeterminism)
        ));
    }
}
