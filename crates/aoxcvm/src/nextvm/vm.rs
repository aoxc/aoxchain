use crate::nextvm::crypto::{CryptoProfile, SignatureEnvelope};
use crate::nextvm::error::NextVmError;
use crate::nextvm::host::{HostAdapter, NullHost};
use crate::nextvm::opcode::{Instruction, Opcode};
use crate::nextvm::state::{Capability, StateStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmConfig {
    pub max_steps: usize,
    pub gas_limit: u64,
    pub crypto_profile: CryptoProfile,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            max_steps: 10_000,
            gas_limit: 1_000_000,
            crypto_profile: CryptoProfile::HybridPqPreferred,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub pc: usize,
    pub opcode: Opcode,
    pub gas_used: u64,
    pub registers: [u64; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub gas_used: u64,
    pub halted: bool,
    pub registers: [u64; 4],
    pub trace: Vec<TraceEvent>,
}

pub struct VmExecution<'a, H: HostAdapter = NullHost> {
    pub config: VmConfig,
    pub state: &'a mut StateStore,
    pub host: H,
}

impl<'a> VmExecution<'a, NullHost> {
    pub fn new(config: VmConfig, state: &'a mut StateStore) -> Self {
        Self {
            config,
            state,
            host: NullHost,
        }
    }
}

impl<'a, H: HostAdapter> VmExecution<'a, H> {
    pub fn run(
        &mut self,
        program: &[Instruction],
        envelope: &SignatureEnvelope,
    ) -> Result<ExecutionOutcome, NextVmError> {
        self.config.crypto_profile.validate_envelope(envelope)?;

        let mut pc = 0usize;
        let mut gas_used = 0u64;
        let mut registers = [0u64; 4];
        let mut steps = 0usize;
        let mut trace = Vec::new();

        while steps < self.config.max_steps {
            let instruction = program.get(pc).ok_or(NextVmError::ProgramCounterOutOfRange)?;
            let next_gas = gas_used.saturating_add(instruction.gas_cost());
            if next_gas > self.config.gas_limit {
                return Err(NextVmError::OutOfGas);
            }
            gas_used = next_gas;

            let mut next_pc = pc + 1;
            match instruction.opcode {
                Opcode::Noop => {}
                Opcode::MovImm => {
                    let register = Self::register_index(instruction.operand_a)?;
                    registers[register] = instruction.operand_b;
                }
                Opcode::Add => {
                    let dst = Self::register_index(instruction.operand_a)?;
                    let src = Self::register_index(instruction.operand_b)?;
                    registers[dst] = registers[dst].saturating_add(registers[src]);
                }
                Opcode::Sub => {
                    let dst = Self::register_index(instruction.operand_a)?;
                    let src = Self::register_index(instruction.operand_b)?;
                    registers[dst] = registers[dst].saturating_sub(registers[src]);
                }
                Opcode::Store => {
                    Self::require(self.state, Capability::StorageWrite)?;
                    let src = Self::register_index(instruction.operand_b)?;
                    self.state.set(instruction.operand_a, registers[src]);
                }
                Opcode::Load => {
                    Self::require(self.state, Capability::StorageRead)?;
                    let dst = Self::register_index(instruction.operand_b)?;
                    registers[dst] = self.state.get(instruction.operand_a);
                }
                Opcode::CallHost => {
                    Self::require(self.state, Capability::HostCall)?;
                    let register = Self::register_index(instruction.operand_a)?;
                    registers[register] = self.host.call(crate::nextvm::host::HostCallRequest {
                        selector: instruction.operand_b,
                        arg0: registers[register],
                    })?;
                }
                Opcode::Checkpoint => {
                    self.state.checkpoint();
                }
                Opcode::Commit => {
                    self.state.commit();
                }
                Opcode::Rollback => {
                    self.state.rollback();
                }
                Opcode::JumpIfZero => {
                    let register = Self::register_index(instruction.operand_a)?;
                    if registers[register] == 0 {
                        next_pc = instruction.operand_b as usize;
                    }
                }
                Opcode::Halt => {
                    trace.push(TraceEvent {
                        pc,
                        opcode: instruction.opcode,
                        gas_used,
                        registers,
                    });
                    return Ok(ExecutionOutcome {
                        gas_used,
                        halted: true,
                        registers,
                        trace,
                    });
                }
            }

            trace.push(TraceEvent {
                pc,
                opcode: instruction.opcode,
                gas_used,
                registers,
            });

            pc = next_pc;
            steps += 1;
        }

        Ok(ExecutionOutcome {
            gas_used,
            halted: false,
            registers,
            trace,
        })
    }

    fn require(state: &StateStore, capability: Capability) -> Result<(), NextVmError> {
        if state.has_capability(capability) {
            Ok(())
        } else {
            Err(NextVmError::MissingCapability(capability.as_str()))
        }
    }

    fn register_index(raw: u64) -> Result<usize, NextVmError> {
        let index = usize::try_from(raw).map_err(|_| NextVmError::InvalidRegisterIndex(raw))?;
        if index < 4 {
            Ok(index)
        } else {
            Err(NextVmError::InvalidRegisterIndex(raw))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_hybrid_envelope() -> SignatureEnvelope {
        SignatureEnvelope {
            classical_sig: vec![1],
            pq_sig: Some(vec![2]),
        }
    }

    #[test]
    fn run_halts_and_updates_registers() {
        let mut state = StateStore::with_capabilities([
            Capability::StorageRead,
            Capability::StorageWrite,
            Capability::HostCall,
        ]);
        let mut execution = VmExecution::new(VmConfig::default(), &mut state);

        let program = vec![
            Instruction::new(Opcode::MovImm, 0, 7),
            Instruction::new(Opcode::MovImm, 1, 9),
            Instruction::new(Opcode::Add, 0, 1),
            Instruction::new(Opcode::CallHost, 0, 1),
            Instruction::new(Opcode::Halt, 0, 0),
        ];

        let outcome = execution.run(&program, &valid_hybrid_envelope()).unwrap();
        assert!(outcome.halted);
        assert_eq!(outcome.registers[0], 17);
        assert_eq!(outcome.trace.len(), 5);
    }

    #[test]
    fn store_requires_capability() {
        let mut state = StateStore::with_capabilities([Capability::StorageRead]);
        let mut execution = VmExecution::new(VmConfig::default(), &mut state);

        let program = vec![
            Instruction::new(Opcode::MovImm, 0, 99),
            Instruction::new(Opcode::Store, 3, 0),
        ];
        let error = execution.run(&program, &valid_hybrid_envelope()).unwrap_err();
        assert_eq!(
            error,
            NextVmError::MissingCapability(Capability::StorageWrite.as_str())
        );
    }

    #[test]
    fn pq_required_profile_rejects_missing_pq_signature() {
        let mut state = StateStore::with_capabilities([]);
        let mut execution = VmExecution::new(
            VmConfig {
                crypto_profile: CryptoProfile::HybridPqRequired,
                ..VmConfig::default()
            },
            &mut state,
        );

        let envelope = SignatureEnvelope {
            classical_sig: vec![1],
            pq_sig: None,
        };

        let error = execution
            .run(&[Instruction::new(Opcode::Halt, 0, 0)], &envelope)
            .unwrap_err();
        assert_eq!(error, NextVmError::InvalidSignatureEnvelope);
    }

    #[test]
    fn checkpoint_rollback_restores_previous_value() {
        let mut state = StateStore::with_capabilities([
            Capability::StorageRead,
            Capability::StorageWrite,
        ]);
        state.set(10, 5);

        let mut execution = VmExecution::new(VmConfig::default(), &mut state);
        let program = vec![
            Instruction::new(Opcode::Checkpoint, 0, 0),
            Instruction::new(Opcode::MovImm, 0, 42),
            Instruction::new(Opcode::Store, 10, 0),
            Instruction::new(Opcode::Rollback, 0, 0),
            Instruction::new(Opcode::Load, 10, 1),
            Instruction::new(Opcode::Halt, 0, 0),
        ];

        let outcome = execution.run(&program, &valid_hybrid_envelope()).unwrap();
        assert_eq!(outcome.registers[1], 5);
    }

    #[test]
    fn jump_if_zero_changes_program_counter() {
        let mut state = StateStore::with_capabilities([]);
        let mut execution = VmExecution::new(VmConfig::default(), &mut state);
        let program = vec![
            Instruction::new(Opcode::MovImm, 0, 0),
            Instruction::new(Opcode::JumpIfZero, 0, 4),
            Instruction::new(Opcode::MovImm, 1, 99),
            Instruction::new(Opcode::Noop, 0, 0),
            Instruction::new(Opcode::MovImm, 1, 7),
            Instruction::new(Opcode::Halt, 0, 0),
        ];

        let outcome = execution.run(&program, &valid_hybrid_envelope()).unwrap();
        assert_eq!(outcome.registers[1], 7);
    }
}
