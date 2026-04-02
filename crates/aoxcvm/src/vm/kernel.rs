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
use crate::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};
use crate::verifier::determinism::{DeterminismError, DeterminismVerifier};
use crate::vm::admission::{AdmissionError, validate_phase1_admission};
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
    Admission(AdmissionError),
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

impl From<AdmissionError> for KernelError {
    fn from(value: AdmissionError) -> Self {
        Self::Admission(value)
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
        let context = self.default_context();
        let tx = Self::default_tx_for_context(&context);
        self.execute_phase1_with_admission(program, context, tx)
    }

    /// Executes and verifies a program using caller-provided immutable execution context
    /// and explicit transaction-envelope admission.
    pub fn execute_phase1_with_context(
        &self,
        program: Program,
        context: ExecutionContext,
        tx: TxEnvelope,
    ) -> Result<KernelOutput, KernelError> {
        let tx = Self::default_tx_for_context(&context);
        self.execute_phase1_with_admission(program, context, tx)
    }

    /// Executes phase-1 flow with explicit transaction-envelope admission.
    pub fn execute_phase1_with_admission(
        &self,
        program: Program,
        context: ExecutionContext,
        tx: TxEnvelope,
    ) -> Result<KernelOutput, KernelError> {
        let limits = DeterminismLimits {
            max_call_depth: self.config.max_call_depth,
            max_gas_limit: self.config.gas_limit,
            min_spec_version: self.config.min_spec_version,
        };

        validate_phase1_admission(&context, &tx, limits, 64 * 1024)?;

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

    fn default_tx_for_context(context: &ExecutionContext) -> TxEnvelope {
        TxEnvelope::new(
            context.environment.chain_id,
            u64::from(context.tx.tx_index),
            TxKind::UserCall,
            FeeBudget::new(context.tx.gas_limit, 1),
            TxPayload::new(vec![0_u8]),
        )
    }

    fn default_context(&self) -> ExecutionContext {
        ExecutionContext::new(
            EnvironmentContext::new(tx.chain_id, 1),
            BlockContext::new(0, 0, 0, [0_u8; 32]),
            TxContext::new([0_u8; 32], 0, tx.fee_budget.gas_limit, false, 1, 0),
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
    use crate::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};
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

        let tx = TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(1_000, 1),
            TxPayload::new(vec![1, 2, 3]),
        );

        let output = kernel.execute_phase1(program, tx).expect("phase1 success");
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

        let tx = TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(500, 1),
            TxPayload::new(vec![1_u8]),
        );

        let err = kernel
            .execute_phase1_with_context(
                Program {
                    code: vec![Instruction::Halt],
                },
                context,
                tx,
            )
            .expect_err("must fail");

        assert!(matches!(
            err,
            KernelError::Admission(crate::vm::admission::AdmissionError::Context(
                crate::context::execution::ContextError::DepthExceedsDeterminism
            ))
        ));
    }

    #[test]
    fn rejects_admission_when_context_and_tx_chain_id_do_not_match() {
        let kernel = AOXCVMachineQX1::new(KernelConfig::default());
        let context = ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 1, 1, [9_u8; 32]),
            TxContext::new([7_u8; 32], 0, 500, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        );

        let tx = TxEnvelope::new(
            2627,
            1,
            TxKind::UserCall,
            FeeBudget::new(500, 1),
            TxPayload::new(vec![1_u8]),
        );

        let err = kernel
            .execute_phase1_with_context(
                Program {
                    code: vec![Instruction::Halt],
                },
                context,
                tx,
            )
            .expect_err("must fail");

        assert!(matches!(
            err,
            KernelError::Admission(crate::vm::admission::AdmissionError::TxValidation(
                crate::tx::validation::ValidationError::ChainIdMismatch
            ))
        ));
    }

    #[test]
    fn rejects_admission_when_context_tx_gas_do_not_match() {
        let kernel = AOXCVMachineQX1::new(KernelConfig::default());
        let context = ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 1, 1, [9_u8; 32]),
            TxContext::new([7_u8; 32], 0, 500, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        );

        let tx = TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(700, 1),
            TxPayload::new(vec![1_u8]),
        );

        let err = kernel
            .execute_phase1_with_context(
                Program {
                    code: vec![Instruction::Halt],
                },
                context,
                tx,
            )
            .expect_err("must fail");

        assert!(matches!(
            err,
            KernelError::Admission(crate::vm::admission::AdmissionError::Context(
                crate::context::execution::ContextError::DepthExceedsDeterminism
            ))
        ));
    }

    #[test]
    fn rejects_admission_when_context_and_tx_chain_id_do_not_match() {
        let kernel = AOXCVMachineQX1::new(KernelConfig::default());
        let context = ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 1, 1, [9_u8; 32]),
            TxContext::new([7_u8; 32], 0, 500, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([1_u8; 32], [2_u8; 32], [1_u8; 32], 0),
        );

        let tx = TxEnvelope::new(
            2627,
            1,
            TxKind::UserCall,
            FeeBudget::new(500, 1),
            TxPayload::new(vec![1_u8]),
        );

        let err = kernel
            .execute_phase1_with_admission(
                Program {
                    code: vec![Instruction::Halt],
                },
                context,
                tx,
            )
            .expect_err("must fail");

        assert!(matches!(
            err,
            KernelError::Admission(crate::vm::admission::AdmissionError::TxValidation(
                crate::tx::validation::ValidationError::ChainIdMismatch
            ))
        ));
    }
}
