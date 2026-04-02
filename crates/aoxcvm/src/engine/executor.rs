//! Deterministic bytecode executor for the phase-3 core prototype.

use crate::bytecode::opcode::{Opcode, OpcodeDecodeError};
use crate::engine::vm::VmState;
use crate::gas::meter::{GasError, GasMeter};
use crate::instructions::arithmetic::{self, ArithmeticTrap};

/// Executor mode used for explicit host behavior boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    ValidateOnly,
    DryRun,
    Commit,
}

/// Canonical trap taxonomy for the prototype executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionTrap {
    DecodeUnknownOpcode(u8),
    DecodeReservedOpcode(u8),
    MalformedImmediate,
    StackUnderflow,
    ArithmeticOverflow,
    DivisionByZero,
    OutOfGas,
}

/// Canonical execution receipt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub mode: ExecutionMode,
    pub gas_used: u64,
    pub halted: bool,
    pub reverted: bool,
    pub trap: Option<ExecutionTrap>,
    pub final_stack: Vec<i64>,
}

/// Deterministic executor implementation.
#[derive(Debug, Clone)]
pub struct Executor {
    mode: ExecutionMode,
    vm: VmState,
    gas: GasMeter,
}

impl Executor {
    pub fn new(mode: ExecutionMode, gas_limit: u64) -> Self {
        Self {
            mode,
            vm: VmState::new(),
            gas: GasMeter::new(gas_limit),
        }
    }

    pub fn execute(mut self, code: &[u8]) -> ExecutionReceipt {
        if self.mode == ExecutionMode::ValidateOnly {
            let trap = self.validate(code).err();
            return self.receipt(trap);
        }

        let mut trap = None;
        while !self.vm.halted() {
            if self.vm.pc() >= code.len() {
                self.vm.halt();
                break;
            }

            let op_byte = code[self.vm.pc()];
            let opcode = match Opcode::from_byte(op_byte) {
                Ok(op) => op,
                Err(OpcodeDecodeError::Unknown(byte)) => {
                    trap = Some(ExecutionTrap::DecodeUnknownOpcode(byte));
                    self.vm.halt();
                    break;
                }
                Err(OpcodeDecodeError::Reserved(byte)) => {
                    trap = Some(ExecutionTrap::DecodeReservedOpcode(byte));
                    self.vm.halt();
                    break;
                }
            };

            if let Err(err) = self.gas.charge(opcode.base_gas()) {
                trap = Some(map_gas_err(err));
                self.vm.halt();
                break;
            }

            match self.step(opcode, code) {
                Ok(()) => {}
                Err(err) => {
                    trap = Some(err);
                    self.vm.halt();
                }
            }
        }

        self.receipt(trap)
    }

    fn validate(&self, code: &[u8]) -> Result<(), ExecutionTrap> {
        let mut pc = 0usize;
        while pc < code.len() {
            let opcode = Opcode::from_byte(code[pc]).map_err(|err| match err {
                OpcodeDecodeError::Unknown(byte) => ExecutionTrap::DecodeUnknownOpcode(byte),
                OpcodeDecodeError::Reserved(byte) => ExecutionTrap::DecodeReservedOpcode(byte),
            })?;
            pc += 1;
            if opcode == Opcode::PushI64 {
                if code.len().saturating_sub(pc) < 8 {
                    return Err(ExecutionTrap::MalformedImmediate);
                }
                pc += 8;
            }
        }
        Ok(())
    }

    fn step(&mut self, opcode: Opcode, code: &[u8]) -> Result<(), ExecutionTrap> {
        match opcode {
            Opcode::Nop => {
                self.vm.advance_pc(1);
                Ok(())
            }
            Opcode::PushI64 => {
                let base = self.vm.pc().saturating_add(1);
                let end = base.saturating_add(8);
                let immediate = code
                    .get(base..end)
                    .ok_or(ExecutionTrap::MalformedImmediate)?;
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(immediate);
                self.vm.push(i64::from_le_bytes(bytes));
                self.vm.set_pc(end);
                Ok(())
            }
            Opcode::Add => self.exec_binary_op(arithmetic::add),
            Opcode::Sub => self.exec_binary_op(arithmetic::sub),
            Opcode::Mul => self.exec_binary_op(arithmetic::mul),
            Opcode::Div => self.exec_binary_op(arithmetic::div),
            Opcode::Mod => self.exec_binary_op(arithmetic::rem),
            Opcode::Halt => {
                self.vm.halt();
                self.vm.advance_pc(1);
                Ok(())
            }
            Opcode::Revert => {
                self.vm.revert();
                self.vm.advance_pc(1);
                Ok(())
            }
        }
    }

    fn exec_binary_op(
        &mut self,
        op: fn(i64, i64) -> Result<i64, ArithmeticTrap>,
    ) -> Result<(), ExecutionTrap> {
        let rhs = self.vm.pop().ok_or(ExecutionTrap::StackUnderflow)?;
        let lhs = self.vm.pop().ok_or(ExecutionTrap::StackUnderflow)?;
        let out = op(lhs, rhs).map_err(map_arithmetic_trap)?;
        self.vm.push(out);
        self.vm.advance_pc(1);
        Ok(())
    }

    fn receipt(&self, trap: Option<ExecutionTrap>) -> ExecutionReceipt {
        ExecutionReceipt {
            mode: self.mode,
            gas_used: self.gas.used(),
            halted: self.vm.halted(),
            reverted: self.vm.reverted(),
            trap,
            final_stack: self.vm.stack().to_vec(),
        }
    }
}

fn map_arithmetic_trap(err: ArithmeticTrap) -> ExecutionTrap {
    match err {
        ArithmeticTrap::StackUnderflow => ExecutionTrap::StackUnderflow,
        ArithmeticTrap::Overflow => ExecutionTrap::ArithmeticOverflow,
        ArithmeticTrap::DivisionByZero => ExecutionTrap::DivisionByZero,
    }
}

fn map_gas_err(err: GasError) -> ExecutionTrap {
    match err {
        GasError::OutOfGas => ExecutionTrap::OutOfGas,
    }
}

#[cfg(test)]
mod tests {
    use super::{ExecutionMode, ExecutionTrap, Executor};
    use crate::bytecode::opcode::Opcode;

    fn push_i64(value: i64) -> Vec<u8> {
        let mut out = vec![Opcode::PushI64.to_byte()];
        out.extend_from_slice(&value.to_le_bytes());
        out
    }

    #[test]
    fn validate_only_rejects_malformed_immediate() {
        let code = vec![Opcode::PushI64.to_byte(), 1, 2];
        let receipt = Executor::new(ExecutionMode::ValidateOnly, 1_000).execute(&code);
        assert_eq!(receipt.trap, Some(ExecutionTrap::MalformedImmediate));
    }

    #[test]
    fn arithmetic_program_executes_deterministically() {
        let mut code = Vec::new();
        code.extend(push_i64(7));
        code.extend(push_i64(2));
        code.push(Opcode::Mul.to_byte());
        code.push(Opcode::Halt.to_byte());

        let receipt_a = Executor::new(ExecutionMode::DryRun, 1_000).execute(&code);
        let receipt_b = Executor::new(ExecutionMode::DryRun, 1_000).execute(&code);
        assert_eq!(receipt_a, receipt_b);
        assert_eq!(receipt_a.final_stack, vec![14]);
        assert_eq!(receipt_a.trap, None);
    }

    #[test]
    fn out_of_gas_is_canonical_trap() {
        let mut code = Vec::new();
        code.extend(push_i64(1));
        code.push(Opcode::Halt.to_byte());

        let receipt = Executor::new(ExecutionMode::Commit, 1).execute(&code);
        assert_eq!(receipt.trap, Some(ExecutionTrap::OutOfGas));
        assert!(receipt.halted);
    }

    #[test]
    fn revert_sets_revert_flag() {
        let code = vec![Opcode::Revert.to_byte()];
        let receipt = Executor::new(ExecutionMode::Commit, 1_000).execute(&code);
        assert!(receipt.reverted);
        assert!(receipt.halted);
    }
}
