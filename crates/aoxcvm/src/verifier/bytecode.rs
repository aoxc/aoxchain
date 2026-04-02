//! Bytecode/program validation for AOXCVM phase-1.

use crate::vm::machine::{Instruction, Program};

/// Bytecode validation errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BytecodeError {
    EmptyProgram,
    MissingHalt,
    ProgramTooLarge { max: usize, got: usize },
    HaltNotTerminal,
    MemoryAccessOutOfBounds {
        offset: usize,
        width: usize,
        max_memory: usize,
    },
    StackUnderflowRisk {
        pc: usize,
        needed: usize,
        available: usize,
    },
    StackOverflowRisk {
        max: usize,
        got: usize,
        pc: usize,
    },
}

/// Deterministic bytecode verifier with bounded limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeVerifier {
    pub max_instructions: usize,
    pub max_memory: usize,
    pub max_stack_depth: usize,
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

        let mut stack_depth = 0usize;
        for (pc, instruction) in program.code.iter().enumerate() {
            let needed = match instruction {
                Instruction::Push(_) | Instruction::LoadMem { .. } | Instruction::Halt => 0,
                Instruction::Add
                | Instruction::Sub
                | Instruction::Mul
                | Instruction::Div
                | Instruction::SStore => 2,
                Instruction::StoreMem { .. } | Instruction::SLoad | Instruction::LogTop => 1,
            };

            if stack_depth < needed {
                return Err(BytecodeError::StackUnderflowRisk {
                    pc,
                    needed,
                    available: stack_depth,
                });
            }

            match instruction {
                Instruction::Push(_) | Instruction::LoadMem { .. } => {
                    stack_depth += 1;
                }
                Instruction::Add | Instruction::Sub | Instruction::Mul | Instruction::Div => {
                    stack_depth -= 1;
                }
                Instruction::StoreMem { .. } | Instruction::SStore => {
                    stack_depth -= needed;
                }
                Instruction::SLoad | Instruction::LogTop | Instruction::Halt => {}
            }

            if stack_depth > self.max_stack_depth {
                return Err(BytecodeError::StackOverflowRisk {
                    max: self.max_stack_depth,
                    got: stack_depth,
                    pc,
                });
            }

            let offset = match instruction {
                Instruction::LoadMem { offset } | Instruction::StoreMem { offset } => Some(*offset),
                _ => None,
            };
            if let Some(offset) = offset {
                let width = core::mem::size_of::<u64>();
                let upper = offset.checked_add(width);
                if upper.is_none_or(|end| end > self.max_memory) {
                    return Err(BytecodeError::MemoryAccessOutOfBounds {
                        offset,
                        width,
                        max_memory: self.max_memory,
                    });
                }
            }
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
            max_memory: 1024,
            max_stack_depth: 64,
        };
        let program = Program {
            code: vec![Instruction::Halt, Instruction::Push(1), Instruction::Halt],
        };
        assert_eq!(
            verifier.verify(&program),
            Err(BytecodeError::HaltNotTerminal)
        );
    }

    #[test]
    fn reject_static_stack_underflow_risk() {
        let verifier = BytecodeVerifier {
            max_instructions: 32,
            max_memory: 1024,
            max_stack_depth: 64,
        };
        let program = Program {
            code: vec![Instruction::Add, Instruction::Halt],
        };
        assert_eq!(
            verifier.verify(&program),
            Err(BytecodeError::StackUnderflowRisk {
                pc: 0,
                needed: 2,
                available: 0,
            })
        );
    }

    #[test]
    fn reject_memory_access_outside_bound() {
        let verifier = BytecodeVerifier {
            max_instructions: 32,
            max_memory: 8,
            max_stack_depth: 64,
        };
        let program = Program {
            code: vec![
                Instruction::Push(7),
                Instruction::StoreMem { offset: 8 },
                Instruction::Halt,
            ],
        };
        assert_eq!(
            verifier.verify(&program),
            Err(BytecodeError::MemoryAccessOutOfBounds {
                offset: 8,
                width: 8,
                max_memory: 8,
            })
        );
    }
}
