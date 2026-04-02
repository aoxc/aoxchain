//! Deterministic phase-1 virtual machine core.

use crate::gas::meter::{GasError, GasMeter};
use crate::memory::heap::{LinearMemory, MemoryError};
use crate::receipts::outcome::{ExecutionReceipt, ReceiptStatus};
use crate::state::{JournaledState, StateError};

/// Deterministic instruction set for phase-1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    /// Pushes immediate value onto stack.
    Push(u64),
    Add,
    Sub,
    Mul,
    Div,
    /// Stores top of stack into memory at `offset`.
    StoreMem {
        offset: usize,
    },
    /// Loads `u64` from memory at `offset` and pushes onto stack.
    LoadMem {
        offset: usize,
    },
    /// Writes key/value from stack into state (`value`, `key` pop order).
    SStore,
    /// Reads state by key from stack and pushes value (or zero).
    SLoad,
    /// Emits a log line with current stack top.
    LogTop,
    Halt,
}

/// Program container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub code: Vec<Instruction>,
}

/// Execution errors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmError {
    OutOfGas,
    StackUnderflow,
    DivisionByZero,
    InvalidProgramCounter,
    MemoryOutOfBounds,
    InvalidCheckpoint,
}

impl From<GasError> for VmError {
    fn from(_: GasError) -> Self {
        Self::OutOfGas
    }
}

impl From<MemoryError> for VmError {
    fn from(_: MemoryError) -> Self {
        Self::MemoryOutOfBounds
    }
}

impl From<StateError> for VmError {
    fn from(_: StateError) -> Self {
        Self::InvalidCheckpoint
    }
}

/// Successful VM execution response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionResult {
    pub receipt: ExecutionReceipt,
    pub stack: Vec<u64>,
    pub final_state: JournaledState,
}

/// Deterministic execution envelope that always carries a receipt.
///
/// `error` is populated when execution fails after admission, while `result`
/// still contains canonical failure receipt material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEnvelope {
    pub result: ExecutionResult,
    pub error: Option<VmError>,
}

/// Deterministic single-threaded VM.
#[derive(Debug, Clone)]
pub struct Machine {
    program: Program,
    pc: usize,
    stack: Vec<u64>,
    memory: LinearMemory,
    gas: GasMeter,
    pub state: JournaledState,
    logs: Vec<String>,
}

impl Machine {
    /// Creates a new machine with empty state.
    pub fn new(program: Program, gas_limit: u64, max_memory: usize) -> Self {
        Self {
            program,
            pc: 0,
            stack: Vec::new(),
            memory: LinearMemory::new(0, max_memory),
            gas: GasMeter::new(gas_limit),
            state: JournaledState::default(),
            logs: Vec::new(),
        }
    }

    /// Creates a machine with a provided starting state.
    pub fn with_state(
        program: Program,
        gas_limit: u64,
        max_memory: usize,
        state: JournaledState,
    ) -> Self {
        let mut vm = Self::new(program, gas_limit, max_memory);
        vm.state = state;
        vm
    }

    /// Executes until `Halt` or error. State writes are reverted on failure.
    pub fn execute(self) -> Result<ExecutionResult, VmError> {
        let envelope = self.execute_enveloped();
        match envelope.error {
            Some(err) => Err(err),
            None => Ok(envelope.result),
        }
    }

    /// Executes and always returns a receipt envelope, including deterministic failures.
    pub fn execute_enveloped(mut self) -> ExecutionEnvelope {
        let checkpoint = self.state.checkpoint();

        loop {
            let Some(instruction) = self.program.code.get(self.pc).cloned() else {
                return self.fail_envelope(VmError::InvalidProgramCounter, checkpoint);
            };
            self.pc += 1;

            if let Instruction::Halt = instruction {
                if let Err(err) = self.gas.charge(Self::gas_cost(&instruction)) {
                    return self.fail_envelope(err.into(), checkpoint);
                }
                if let Err(err) = self.state.commit(checkpoint) {
                    return self.fail_envelope(err.into(), checkpoint);
                }

                let receipt = ExecutionReceipt::from_state(
                    ReceiptStatus::Success,
                    self.gas.used(),
                    self.logs,
                    &self.state,
                );
                return ExecutionEnvelope {
                    result: ExecutionResult {
                        receipt,
                        stack: self.stack,
                        final_state: self.state,
                    },
                    error: None,
                };
            }

            if let Err(err) = self.step(instruction) {
                return self.fail_envelope(err, checkpoint);
            }
        }
    }

    fn fail_envelope(&mut self, err: VmError, checkpoint: usize) -> ExecutionEnvelope {
        let _ = self.state.rollback(checkpoint);
        let receipt = ExecutionReceipt::from_state(
            ReceiptStatus::Failed,
            self.gas.used(),
            self.logs.clone(),
            &self.state,
        );
        ExecutionEnvelope {
            result: ExecutionResult {
                receipt,
                stack: self.stack.clone(),
                final_state: self.state.clone(),
            },
            error: Some(err),
        }
    }

    fn step(&mut self, instruction: Instruction) -> Result<(), VmError> {
        self.gas.charge(Self::gas_cost(&instruction))?;

        match instruction {
            Instruction::Push(v) => self.stack.push(v),
            Instruction::Add => {
                let (a, b) = self.pop2()?;
                self.stack.push(a.wrapping_add(b));
            }
            Instruction::Sub => {
                let (a, b) = self.pop2()?;
                self.stack.push(a.wrapping_sub(b));
            }
            Instruction::Mul => {
                let (a, b) = self.pop2()?;
                self.stack.push(a.wrapping_mul(b));
            }
            Instruction::Div => {
                let (a, b) = self.pop2()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(a / b);
            }
            Instruction::StoreMem { offset } => {
                let value = self.pop1()?;
                self.memory.write_u64(offset, value)?;
            }
            Instruction::LoadMem { offset } => {
                let value = self.memory.read_u64(offset)?;
                self.stack.push(value);
            }
            Instruction::SStore => {
                let value = self.pop1()?;
                let key = self.pop1()?;
                self.state
                    .put(key.to_le_bytes().to_vec(), value.to_le_bytes().to_vec());
            }
            Instruction::SLoad => {
                let key = self.pop1()?;
                let value = self
                    .state
                    .get(&key.to_le_bytes())
                    .map(|bytes| {
                        let mut arr = [0_u8; 8];
                        let copy_len = bytes.len().min(8);
                        arr[..copy_len].copy_from_slice(&bytes[..copy_len]);
                        u64::from_le_bytes(arr)
                    })
                    .unwrap_or(0);
                self.stack.push(value);
            }
            Instruction::LogTop => {
                let top = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                self.logs.push(format!("top={top}"));
            }
            Instruction::Halt => unreachable!("handled in execute loop"),
        }

        Ok(())
    }

    fn pop1(&mut self) -> Result<u64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn pop2(&mut self) -> Result<(u64, u64), VmError> {
        let b = self.pop1()?;
        let a = self.pop1()?;
        Ok((a, b))
    }

    fn gas_cost(instruction: &Instruction) -> u64 {
        match instruction {
            Instruction::Push(_) => 1,
            Instruction::Add | Instruction::Sub => 2,
            Instruction::Mul | Instruction::Div => 3,
            Instruction::StoreMem { .. } | Instruction::LoadMem { .. } => 4,
            Instruction::SStore => 20,
            Instruction::SLoad => 8,
            Instruction::LogTop => 5,
            Instruction::Halt => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Instruction, Machine, Program, VmError};
    use crate::receipts::outcome::ReceiptStatus;

    #[test]
    fn deterministic_execution_matches_between_runs() {
        let program = Program {
            code: vec![
                Instruction::Push(7),
                Instruction::Push(9),
                Instruction::Add,
                Instruction::Push(1),
                Instruction::SStore,
                Instruction::Push(1),
                Instruction::SLoad,
                Instruction::Halt,
            ],
        };

        let a = Machine::new(program.clone(), 200, 1024)
            .execute()
            .expect("run A");
        let b = Machine::new(program, 200, 1024).execute().expect("run B");

        assert_eq!(a.stack, b.stack);
        assert_eq!(a.receipt.state_root, b.receipt.state_root);
        assert_eq!(a.receipt.gas_used, b.receipt.gas_used);
    }

    #[test]
    fn rollback_on_failure() {
        let program = Program {
            code: vec![
                Instruction::Push(1),
                Instruction::Push(99),
                Instruction::SStore,
                Instruction::Push(1),
                Instruction::Push(0),
                Instruction::Div,
                Instruction::Halt,
            ],
        };

        let err = Machine::new(program, 200, 1024)
            .execute()
            .expect_err("must fail");
        assert_eq!(err, VmError::DivisionByZero);
    }

    #[test]
    fn execute_enveloped_reports_failed_receipt() {
        let program = Program {
            code: vec![Instruction::Push(1), Instruction::Push(0), Instruction::Div],
        };
        let envelope = Machine::new(program, 50, 128).execute_enveloped();
        assert_eq!(envelope.error, Some(VmError::DivisionByZero));
        assert_eq!(envelope.result.receipt.status, ReceiptStatus::Failed);
    }
}
