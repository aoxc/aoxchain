//! Bytecode/program validation for AOXCVM phase-1.

use crate::vm::machine::{Instruction, Program};

/// Bytecode validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytecodeError {
    EmptyProgram,
    MissingHalt,
    ProgramTooLarge { max: usize, got: usize },
    HaltNotTerminal,
}

/// Deterministic bytecode verifier with bounded limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeVerifier {
    pub max_instructions: usize,
}

impl BytecodeVerifier {
    /// Validates basic determinism constraints for phase-1 programs.
    pub fn verify(&self, program: &Program) -> Result<(), BytecodeError> {
        if program.code.is_empty() {
            return Err(BytecodeError::EmptyProgram);
        }
        if program.code.len() > self.max_instructions {
            return Err(BytecodeError::ProgramTooLarge {
                max: self.max_instructions,
                got: program.code.len(),
            });
        }
        if !matches!(program.code.last(), Some(Instruction::Halt)) {
            return Err(BytecodeError::MissingHalt);
        }
        if program.code[..program.code.len() - 1]
            .iter()
            .any(|op| matches!(op, Instruction::Halt))
        {
            return Err(BytecodeError::HaltNotTerminal);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BytecodeError, BytecodeVerifier};
    use crate::vm::machine::{Instruction, Program};

    #[test]
    fn reject_mid_program_halt() {
        let verifier = BytecodeVerifier {
            max_instructions: 32,
        };
        let program = Program {
            code: vec![Instruction::Halt, Instruction::Push(1), Instruction::Halt],
        };
        assert_eq!(
            verifier.verify(&program),
            Err(BytecodeError::HaltNotTerminal)
        );
    }
}
