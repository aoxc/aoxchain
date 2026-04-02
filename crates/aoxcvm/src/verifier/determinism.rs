//! Deterministic execution verifier for AOXCVM phase-1.

use crate::receipts::commitment::ReceiptCommitment;
use crate::receipts::outcome::ReceiptStatus;
use crate::receipts::proof::ReceiptProof;
use crate::verifier::bytecode::{BytecodeError, BytecodeVerifier};
use crate::verifier::invariants::{InvariantError, InvariantVerifier};
use crate::vm::machine::{ExecutionResult, Instruction, Machine, Program, VmError};

/// Verification errors for deterministic execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeterminismError {
    /// Program includes unsupported / unsafe instruction pattern.
    NonDeterministicProgram(&'static str),
    /// VM execution failed while producing witness receipt.
    ExecutionFailed(VmError),
    /// Replay does not match witness output.
    ReceiptMismatch,
    /// Bytecode does not satisfy deterministic admission rules.
    InvalidBytecode(BytecodeError),
    /// Runtime invariants are violated.
    InvariantViolation(InvariantError),
}

/// Phase-1 deterministic verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterminismVerifier {
    pub gas_limit: u64,
    pub max_memory: usize,
}

impl DeterminismVerifier {
    /// Static instruction-level checks before execution.
    pub fn validate_program(&self, program: &Program) -> Result<(), DeterminismError> {
        if program.code.is_empty() {
            return Err(DeterminismError::NonDeterministicProgram(
                "program cannot be empty",
            ));
        }
        if !matches!(program.code.last(), Some(Instruction::Halt)) {
            return Err(DeterminismError::NonDeterministicProgram(
                "program must terminate with HALT",
            ));
        }

        BytecodeVerifier {
            max_instructions: 4096,
        }
        .verify(program)
        .map_err(DeterminismError::InvalidBytecode)
    }

    /// Executes once and verifies deterministic replay equivalence.
    pub fn verify(&self, program: Program) -> Result<ExecutionResult, DeterminismError> {
        self.validate_program(&program)?;

        let first = Machine::new(program.clone(), self.gas_limit, self.max_memory)
            .execute()
            .map_err(DeterminismError::ExecutionFailed)?;
        let second = Machine::new(program, self.gas_limit, self.max_memory)
            .execute()
            .map_err(DeterminismError::ExecutionFailed)?;

        if first.receipt.status != ReceiptStatus::Success
            || first.receipt != second.receipt
            || first.stack != second.stack
        {
            return Err(DeterminismError::ReceiptMismatch);
        }

        let proof = ReceiptProof::new(&first.receipt, 2);
        if !proof.verify_receipt(&second.receipt) {
            return Err(DeterminismError::ReceiptMismatch);
        }

        let first_commitment = ReceiptCommitment::from_receipt(&first.receipt);
        let second_commitment = ReceiptCommitment::from_receipt(&second.receipt);
        if first_commitment != second_commitment {
            return Err(DeterminismError::ReceiptMismatch);
        }

        InvariantVerifier::verify(&first, self.gas_limit)
            .map_err(DeterminismError::InvariantViolation)?;

        Ok(first)
    }
}

#[cfg(test)]
mod tests {
    use super::{DeterminismError, DeterminismVerifier};
    use crate::vm::machine::{Instruction, Program};

    #[test]
    fn rejects_program_without_halt() {
        let verifier = DeterminismVerifier {
            gas_limit: 100,
            max_memory: 1024,
        };
        let program = Program {
            code: vec![Instruction::Push(1)],
        };
        assert_eq!(
            verifier.verify(program),
            Err(DeterminismError::NonDeterministicProgram(
                "program must terminate with HALT"
            ))
        );
    }

    #[test]
    fn verifies_simple_program() {
        let verifier = DeterminismVerifier {
            gas_limit: 100,
            max_memory: 1024,
        };
        let program = Program {
            code: vec![
                Instruction::Push(2),
                Instruction::Push(3),
                Instruction::Mul,
                Instruction::Halt,
            ],
        };

        let result = verifier.verify(program).expect("must verify");
        assert_eq!(result.stack, vec![6]);
    }
}
