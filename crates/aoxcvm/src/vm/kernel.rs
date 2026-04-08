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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelConfig {
    pub gas_limit: u64,
    pub max_memory: usize,
    pub max_stack_depth: usize,
    pub max_call_depth: u16,
    pub min_spec_version: u32,
    pub max_payload_bytes: usize,
    pub security_level: KernelSecurityLevel,
}

/// Security posture selector for kernel baseline presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelSecurityLevel {
    #[deprecated(note = "legacy-only profile; use Quantum for new deployments")]
    Standard,
    Quantum,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self::quantum_default()
    }
}

impl KernelConfig {
    /// Returns the canonical quantum-first default profile.
    pub const fn quantum_default() -> Self {
        Self::for_security_level(KernelSecurityLevel::Quantum)
    }

    /// Returns a conservative preset for deployments that want a stricter
    /// quantum-readiness baseline.
    #[allow(deprecated)]
    pub const fn for_security_level(level: KernelSecurityLevel) -> Self {
        match level {
            KernelSecurityLevel::Standard => Self {
                gas_limit: 1_000_000,
                max_memory: 1024 * 1024,
                max_stack_depth: 1024,
                max_call_depth: 64,
                min_spec_version: 1,
                max_payload_bytes: 64 * 1024,
                security_level: KernelSecurityLevel::Standard,
            },
            KernelSecurityLevel::Quantum => Self {
                gas_limit: 750_000,
                max_memory: 768 * 1024,
                max_stack_depth: 768,
                max_call_depth: 48,
                min_spec_version: 2,
                max_payload_bytes: 32 * 1024,
                security_level: KernelSecurityLevel::Quantum,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelOutput {
    pub context: ExecutionContext,
    pub result: ExecutionResult,
    pub receipt_proof: ReceiptProof,
}

impl KernelOutput {
    pub fn final_state(&self) -> &JournaledState {
        &self.result.final_state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    Context(ContextError),
    Admission(AdmissionError),
    Determinism(DeterminismError),
    ConfigInvariant(&'static str),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AOXCVMachineQX1 {
    config: KernelConfig,
}

impl AOXCVMachineQX1 {
    pub fn new(config: KernelConfig) -> Self {
        Self { config }
    }

    pub fn execute_phase1(&self, program: Program) -> Result<KernelOutput, KernelError> {
        let context = self.default_context();
        let tx = Self::default_tx_for_context(&context);
        self.execute_phase1_with_admission(program, context, tx)
    }

    pub fn execute_phase1_with_context(
        &self,
        program: Program,
        context: ExecutionContext,
        tx: TxEnvelope,
    ) -> Result<KernelOutput, KernelError> {
        self.execute_phase1_with_admission(program, context, tx)
    }

    pub fn execute_phase1_with_admission(
        &self,
        program: Program,
        context: ExecutionContext,
        tx: TxEnvelope,
    ) -> Result<KernelOutput, KernelError> {
        self.validate_security_invariants()?;

        let limits = DeterminismLimits {
            max_call_depth: self.config.max_call_depth,
            max_gas_limit: self.config.gas_limit,
            min_spec_version: self.config.min_spec_version,
        };

        validate_phase1_admission(&context, &tx, limits, self.config.max_payload_bytes)?;

        let verifier = DeterminismVerifier {
            gas_limit: context.tx.gas_limit,
            max_memory: self.config.max_memory,
            max_stack_depth: self.config.max_stack_depth,
        };

        let result = verifier.verify(program)?;
        let receipt_proof = ReceiptProof::new(&result.receipt, 2);
        Ok(KernelOutput {
            context,
            result,
            receipt_proof,
        })
    }

    fn validate_security_invariants(&self) -> Result<(), KernelError> {
        if self.config.security_level == KernelSecurityLevel::Quantum {
            let quantum = KernelConfig::for_security_level(KernelSecurityLevel::Quantum);
            if self.config.gas_limit > quantum.gas_limit {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires gas_limit <= 750_000",
                ));
            }
            if self.config.max_memory > quantum.max_memory {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires max_memory <= 768KiB",
                ));
            }
            if self.config.max_stack_depth > quantum.max_stack_depth {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires max_stack_depth <= 768",
                ));
            }
            if self.config.max_call_depth > quantum.max_call_depth {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires max_call_depth <= 48",
                ));
            }
            if self.config.min_spec_version < quantum.min_spec_version {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires min_spec_version >= 2",
                ));
            }
            if self.config.max_payload_bytes > quantum.max_payload_bytes {
                return Err(KernelError::ConfigInvariant(
                    "quantum profile requires max_payload_bytes <= 32KiB",
                ));
            }
        }
        Ok(())
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
            EnvironmentContext::new(2626, 1),
            BlockContext::new(0, 0, 0, [0_u8; 32]),
            TxContext::new([0_u8; 32], 0, self.config.gas_limit, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([0_u8; 32], [0_u8; 32], [0_u8; 32], 0),
        )
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::{AOXCVMachineQX1, KernelConfig, KernelError, KernelSecurityLevel};
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
            max_stack_depth: 64,
            max_call_depth: 64,
            min_spec_version: 1,
            max_payload_bytes: 64 * 1024,
            security_level: KernelSecurityLevel::Standard,
        });

        let program = Program {
            code: vec![
                Instruction::Push(1),
                Instruction::Push(11),
                Instruction::SStore,
                Instruction::Push(1),
                Instruction::SLoad,
                Instruction::Halt,
            ],
        };

        let output = kernel.execute_phase1(program).expect("phase1 success");
        assert_eq!(output.result.stack, vec![11]);
        assert!(output.receipt_proof.verify_receipt(&output.result.receipt));
    }

    #[test]
    fn rejects_context_depth_over_limit() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 1000,
            max_memory: 1024,
            max_stack_depth: 64,
            max_call_depth: 4,
            min_spec_version: 1,
            max_payload_bytes: 64 * 1024,
            security_level: KernelSecurityLevel::Standard,
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
    fn quantum_security_profile_is_stricter_than_standard() {
        let standard = KernelConfig::for_security_level(KernelSecurityLevel::Standard);
        let quantum = KernelConfig::for_security_level(KernelSecurityLevel::Quantum);

        assert!(quantum.gas_limit < standard.gas_limit);
        assert!(quantum.max_memory < standard.max_memory);
        assert!(quantum.max_stack_depth < standard.max_stack_depth);
        assert!(quantum.max_call_depth < standard.max_call_depth);
        assert!(quantum.min_spec_version > standard.min_spec_version);
    }

    #[test]
    fn default_kernel_config_is_quantum_baseline() {
        let default = KernelConfig::default();
        let quantum = KernelConfig::for_security_level(KernelSecurityLevel::Quantum);

        assert_eq!(default, quantum);
        assert_eq!(default.security_level, KernelSecurityLevel::Quantum);
    }

    #[test]
    fn quantum_profile_rejects_weaker_custom_limits() {
        let kernel = AOXCVMachineQX1::new(KernelConfig {
            gas_limit: 900_000,
            max_memory: 768 * 1024,
            max_stack_depth: 768,
            max_call_depth: 48,
            min_spec_version: 2,
            max_payload_bytes: 32 * 1024,
            security_level: KernelSecurityLevel::Quantum,
        });

        let err = kernel
            .execute_phase1(Program {
                code: vec![Instruction::Halt],
            })
            .expect_err("must fail when quantum limits are weakened");

        assert!(matches!(err, KernelError::ConfigInvariant(_)));
    }
}
