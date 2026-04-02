//! Deterministic execution verifier for AOXCVM phase-1.

use crate::receipts::commitment::ReceiptCommitment;
use crate::receipts::proof::ReceiptProof;
use crate::verifier::bytecode::{BytecodeError, BytecodeVerifier};
use crate::verifier::invariants::{InvariantError, InvariantVerifier};
use crate::vm::machine::{
    ExecutionEnvelope, ExecutionResult, Instruction, Machine, Program, VmError,
};

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
            max_memory: self.max_memory,
            max_stack_depth: 1024,
        }
        .verify(program)
        .map_err(DeterminismError::InvalidBytecode)
    }

    /// Executes once and verifies deterministic replay equivalence.
    pub fn verify(&self, program: Program) -> Result<ExecutionResult, DeterminismError> {
        let envelope = self.verify_enveloped(program)?;
        match envelope.error {
            Some(err) => Err(DeterminismError::ExecutionFailed(err)),
            None => Ok(envelope.result),
        }
    }

    /// Executes twice and returns deterministic envelope (including failure receipts).
    pub fn verify_enveloped(
        &self,
        program: Program,
    ) -> Result<ExecutionEnvelope, DeterminismError> {
        self.validate_program(&program)?;

        let first =
            Machine::new(program.clone(), self.gas_limit, self.max_memory).execute_enveloped();
        let second = Machine::new(program, self.gas_limit, self.max_memory).execute_enveloped();

        if first.error != second.error
            || first.result.receipt != second.result.receipt
            || first.result.stack != second.result.stack
        {
            return Err(DeterminismError::ReceiptMismatch);
        }

        let proof = ReceiptProof::new(&first.result.receipt, 2);
        if !proof.verify_receipt(&second.result.receipt) {
            return Err(DeterminismError::ReceiptMismatch);
        }

        let first_commitment = ReceiptCommitment::from_receipt(&first.result.receipt);
        let second_commitment = ReceiptCommitment::from_receipt(&second.result.receipt);
        if first_commitment != second_commitment {
            return Err(DeterminismError::ReceiptMismatch);
        }

        if first.error.is_none() {
            InvariantVerifier::verify(&first.result, self.gas_limit)
                .map_err(DeterminismError::InvariantViolation)?;
        }

        Ok(first)
    }
}

#[cfg(test)]
mod tests {
    use super::{DeterminismError, DeterminismVerifier};
    use crate::receipts::outcome::ReceiptStatus;
    use crate::vm::machine::{Instruction, Program, VmError};

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

    #[test]
    fn deterministic_failure_returns_failed_receipt() {
        let verifier = DeterminismVerifier {
            gas_limit: 100,
            max_memory: 1024,
        };
        let program = Program {
            code: vec![
                Instruction::Push(1),
                Instruction::Push(0),
                Instruction::Div,
                Instruction::Halt,
            ],
        };

        let envelope = verifier
            .verify_enveloped(program)
            .expect("deterministic trap");
        assert_eq!(envelope.error, Some(VmError::DivisionByZero));
        assert_eq!(envelope.result.receipt.status, ReceiptStatus::Failed);
    }
}
