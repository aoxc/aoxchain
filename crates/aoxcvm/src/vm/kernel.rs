//! AOXC-VMachine-QX1 phase-1 kernel orchestration.

use crate::receipts::proof::ReceiptProof;
use crate::state::JournaledState;
use crate::verifier::determinism::{DeterminismError, DeterminismVerifier};
use crate::vm::machine::{ExecutionEnvelope, Program};

/// Configuration for a phase-1 kernel run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelConfig {
    pub gas_limit: u64,
    pub max_memory: usize,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            gas_limit: 1_000_000,
            max_memory: 1024 * 1024,
        }
    }
}

/// Minimal full phase-1 output surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelOutput {
    pub envelope: ExecutionEnvelope,
    pub receipt_proof: ReceiptProof,
}

impl KernelOutput {
    /// Returns final deterministic state snapshot source.
    pub fn final_state(&self) -> &JournaledState {
        &self.envelope.result.final_state
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

    /// Executes and verifies phase-1 program determinism, including failed receipts.
    pub fn execute_phase1(&self, program: Program) -> Result<KernelOutput, DeterminismError> {
        let verifier = DeterminismVerifier {
            gas_limit: self.config.gas_limit,
            max_memory: self.config.max_memory,
        };

        let envelope = verifier.verify_enveloped(program)?;
        let receipt_proof = ReceiptProof::new(&envelope.result.receipt, 2);
        Ok(KernelOutput {
            envelope,
            receipt_proof,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AOXCVMachineQX1, KernelConfig};
    use crate::receipts::outcome::ReceiptStatus;
    use crate::vm::machine::{Instruction, Program, VmError};

    #[test]
    fn phase1_kernel_executes_full_minimal_surface() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 1_000,
            max_memory: 1024,
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
        assert_eq!(output.envelope.error, None);
        assert_eq!(output.envelope.result.stack, vec![11]);
        assert!(
            output
                .receipt_proof
                .verify_receipt(&output.envelope.result.receipt)
        );
        assert_eq!(
            output.final_state().get(&1_u64.to_le_bytes()),
            Some(&11_u64.to_le_bytes()[..])
        );
    }

    #[test]
    fn phase1_kernel_produces_failed_receipt_deterministically() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 1_000,
            max_memory: 1024,
        });

        let program = Program {
            code: vec![
                Instruction::Push(8),
                Instruction::Push(0),
                Instruction::Div,
                Instruction::Halt,
            ],
        };

        let output = kernel
            .execute_phase1(program)
            .expect("deterministic failure envelope");
        assert_eq!(output.envelope.error, Some(VmError::DivisionByZero));
        assert_eq!(output.envelope.result.receipt.status, ReceiptStatus::Failed);
        assert!(
            output
                .receipt_proof
                .verify_receipt(&output.envelope.result.receipt)
        );
    }
}
