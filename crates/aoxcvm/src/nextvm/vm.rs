use crate::nextvm::crypto::{CryptoProfile, SignatureEnvelope};
use crate::nextvm::error::NextVmError;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub gas_used: u64,
    pub halted: bool,
    pub accumulator: u64,
}

pub struct VmExecution<'a> {
    pub config: VmConfig,
    pub state: &'a mut StateStore,
}

impl<'a> VmExecution<'a> {
    pub fn run(
        &mut self,
        program: &[Instruction],
        envelope: &SignatureEnvelope,
    ) -> Result<ExecutionOutcome, NextVmError> {
        self.config.crypto_profile.validate_envelope(envelope)?;

        let mut pc = 0usize;
        let mut gas_used = 0u64;
        let mut accumulator = 0u64;
        let mut steps = 0usize;

        while steps < self.config.max_steps {
            let instruction = program.get(pc).ok_or(NextVmError::ProgramCounterOutOfRange)?;
            let next_gas = gas_used.saturating_add(instruction.gas_cost());
            if next_gas > self.config.gas_limit {
                return Err(NextVmError::OutOfGas);
            }
            gas_used = next_gas;

            match instruction.opcode {
                Opcode::Noop => {}
                Opcode::Add => {
                    accumulator = instruction.operand_a.saturating_add(instruction.operand_b);
                }
                Opcode::Sub => {
                    accumulator = instruction.operand_a.saturating_sub(instruction.operand_b);
                }
                Opcode::Store => {
                    if !self.state.has_capability(Capability::StorageWrite) {
                        return Err(NextVmError::MissingCapability(
                            Capability::StorageWrite.as_str(),
                        ));
                    }
                    self.state.set(instruction.operand_a, instruction.operand_b);
                }
                Opcode::Load => {
                    if !self.state.has_capability(Capability::StorageRead) {
                        return Err(NextVmError::MissingCapability(
                            Capability::StorageRead.as_str(),
                        ));
                    }
                    accumulator = self.state.get(instruction.operand_a);
                }
                Opcode::CallHost => {
                    if !self.state.has_capability(Capability::HostCall) {
                        return Err(NextVmError::MissingCapability(Capability::HostCall.as_str()));
                    }
                    accumulator = accumulator.saturating_add(1);
                }
                Opcode::Halt => {
                    return Ok(ExecutionOutcome {
                        gas_used,
                        halted: true,
                        accumulator,
                    });
                }
            }

            pc += 1;
            steps += 1;
        }

        Ok(ExecutionOutcome {
            gas_used,
            halted: false,
            accumulator,
        })
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
    fn run_halts_and_updates_accumulator() {
        let mut state = StateStore::with_capabilities([
            Capability::StorageRead,
            Capability::StorageWrite,
            Capability::HostCall,
        ]);
        let mut execution = VmExecution {
            config: VmConfig::default(),
            state: &mut state,
        };

        let program = vec![
            Instruction::new(Opcode::Add, 7, 9),
            Instruction::new(Opcode::CallHost, 0, 0),
            Instruction::new(Opcode::Halt, 0, 0),
        ];

        let outcome = execution.run(&program, &valid_hybrid_envelope()).unwrap();
        assert!(outcome.halted);
        assert_eq!(outcome.accumulator, 17);
    }

    #[test]
    fn store_requires_capability() {
        let mut state = StateStore::with_capabilities([Capability::StorageRead]);
        let mut execution = VmExecution {
            config: VmConfig::default(),
            state: &mut state,
        };

        let program = vec![Instruction::new(Opcode::Store, 3, 99)];
        let error = execution.run(&program, &valid_hybrid_envelope()).unwrap_err();
        assert_eq!(
            error,
            NextVmError::MissingCapability(Capability::StorageWrite.as_str())
        );
    }

    #[test]
    fn pq_required_profile_rejects_missing_pq_signature() {
        let mut state = StateStore::with_capabilities([]);
        let mut execution = VmExecution {
            config: VmConfig {
                crypto_profile: CryptoProfile::HybridPqRequired,
                ..VmConfig::default()
            },
            state: &mut state,
        };

        let envelope = SignatureEnvelope {
            classical_sig: vec![1],
            pq_sig: None,
        };

        let error = execution
            .run(&[Instruction::new(Opcode::Halt, 0, 0)], &envelope)
            .unwrap_err();
        assert_eq!(error, NextVmError::InvalidSignatureEnvelope);
    }
}
