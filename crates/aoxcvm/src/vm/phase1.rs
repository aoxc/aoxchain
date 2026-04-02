//! Canonical phase-1 execution entrypoint and contracts.

use crate::auth::envelope::AuthEnvelope;
use crate::context::execution::{ContextError, ExecutionContext};
use crate::receipts::outcome::ExecutionReceipt;
use crate::state::JournaledState;
use crate::tx::envelope::TxEnvelope;
use crate::vm::machine::{Machine, Program, VmError};
use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{ContractDescriptor, VmTarget};

/// Stable execution context contract.
pub type ExecutionContractContext = ExecutionContext;
/// Stable receipt contract.
pub type Receipt = ExecutionReceipt;

/// Canonical VM spec used by the phase-1 kernel entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmSpec {
    pub gas_limit: u64,
    pub max_memory: usize,
    pub max_object_bytes: usize,
}

/// Configuration-time spec derivation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecError {
    VmTargetDisabledByConfig,
}

impl VmSpec {
    /// Build a phase-1 VM spec from contract-policy config with fail-closed target checks.
    pub fn from_config(
        config: &ContractsConfig,
        descriptor: &ContractDescriptor,
    ) -> Result<Self, SpecError> {
        let target_enabled = config
            .artifact_policy
            .allowed_vm_targets
            .iter()
            .any(|target| {
                matches!(
                    (target, &descriptor.manifest.vm_target),
                    (VmTarget::Wasm, VmTarget::Wasm)
                        | (VmTarget::Evm, VmTarget::Evm)
                        | (VmTarget::SuiLike, VmTarget::SuiLike)
                        | (VmTarget::Custom(_), VmTarget::Custom(_))
                )
            });

        if !target_enabled {
            return Err(SpecError::VmTargetDisabledByConfig);
        }

        Ok(Self::default())
    }
}

impl Default for VmSpec {
    fn default() -> Self {
        Self {
            gas_limit: 1_000_000,
            max_memory: 1024 * 1024,
            max_object_bytes: 64 * 1024,
        }
    }
}

/// Input contract for canonical execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionContract {
    pub tx: TxEnvelope,
    pub auth: AuthEnvelope,
    pub object: Vec<u8>,
    pub context: ExecutionContractContext,
    pub program: Program,
}

/// Canonical execution output contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub receipt: Receipt,
    pub stack: Vec<u64>,
    pub vm_error: Option<VmError>,
}

/// Pre-execution admission errors (strictly outside VM execution).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    InvalidAuth,
    InvalidObject,
    MalformedInput,
    Context(ContextError),
}

impl From<ContextError> for AdmissionError {
    fn from(value: ContextError) -> Self {
        Self::Context(value)
    }
}

/// Canonical phase-1 execution error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    Admission(AdmissionError),
    Host,
}

impl From<ContextError> for ExecuteError {
    fn from(value: ContextError) -> Self {
        Self::Admission(AdmissionError::Context(value))
    }
}

/// Host boundary: VM cannot persist state directly.
pub trait Host {
    fn load_state(&self) -> JournaledState;
    fn checkpoint(&mut self) -> Result<usize, ()>;
    fn rollback(&mut self, checkpoint: usize) -> Result<(), ()>;
    fn commit(&mut self, checkpoint: usize, state: JournaledState) -> Result<(), ()>;
}

/// Authentication admission boundary.
pub trait AuthVerifier {
    fn verify(&self, tx: &TxEnvelope, auth: &AuthEnvelope) -> bool;
}

/// Object/bytecode admission boundary.
pub trait ObjectVerifier {
    fn verify(&self, object: &[u8], program: &Program, spec: VmSpec) -> bool;
}

/// Canonical phase-1 kernel entry.
///
/// The ordering is strict:
/// 1. auth verify,
/// 2. object/bytecode verify,
/// 3. host checkpoint,
/// 4. VM execute,
/// 5. host rollback|commit.
pub fn execute(
    contract: &ExecutionContract,
    host: &mut impl Host,
    spec: VmSpec,
    auth_verifier: &impl AuthVerifier,
    object_verifier: &impl ObjectVerifier,
) -> Result<ExecutionOutcome, ExecuteError> {
    if contract.tx.payload.is_empty() {
        return Err(ExecuteError::Admission(AdmissionError::MalformedInput));
    }

    contract
        .context
        .validate(crate::context::deterministic::DeterminismLimits {
            max_call_depth: 64,
            max_gas_limit: spec.gas_limit,
            min_spec_version: 1,
        })?;

    if !auth_verifier.verify(&contract.tx, &contract.auth) {
        return Err(ExecuteError::Admission(AdmissionError::InvalidAuth));
    }

    if contract.object.len() > spec.max_object_bytes
        || !object_verifier.verify(&contract.object, &contract.program, spec)
    {
        return Err(ExecuteError::Admission(AdmissionError::InvalidObject));
    }

    let checkpoint = host.checkpoint().map_err(|_| ExecuteError::Host)?;
    let initial_state = host.load_state();
    let envelope = Machine::with_state(
        contract.program.clone(),
        contract.tx.fee_budget.gas_limit,
        spec.max_memory,
        initial_state,
    )
    .execute_enveloped();

    match envelope.error {
        Some(err) => {
            host.rollback(checkpoint).map_err(|_| ExecuteError::Host)?;
            Ok(ExecutionOutcome {
                receipt: envelope.result.receipt,
                stack: envelope.result.stack,
                vm_error: Some(err),
            })
        }
        None => {
            host.commit(checkpoint, envelope.result.final_state)
                .map_err(|_| ExecuteError::Host)?;
            Ok(ExecutionOutcome {
                receipt: envelope.result.receipt,
                stack: envelope.result.stack,
                vm_error: None,
            })
        }
    }
}

/// Minimal in-memory host implementation for deterministic tests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryHost {
    state: JournaledState,
    checkpoints: Vec<JournaledState>,
}

impl Host for InMemoryHost {
    fn load_state(&self) -> JournaledState {
        self.state.clone()
    }

    fn checkpoint(&mut self) -> Result<usize, ()> {
        self.checkpoints.push(self.state.clone());
        Ok(self.checkpoints.len() - 1)
    }

    fn rollback(&mut self, checkpoint: usize) -> Result<(), ()> {
        let state = self.checkpoints.get(checkpoint).cloned().ok_or(())?;
        self.state = state;
        self.checkpoints.truncate(checkpoint);
        Ok(())
    }

    fn commit(&mut self, checkpoint: usize, state: JournaledState) -> Result<(), ()> {
        if checkpoint >= self.checkpoints.len() {
            return Err(());
        }
        self.state = state;
        self.checkpoints.truncate(checkpoint);
        Ok(())
    }
}

/// Baseline verifier that enforces envelope-level auth shape constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicAuthVerifier;

impl AuthVerifier for BasicAuthVerifier {
    fn verify(&self, _tx: &TxEnvelope, auth: &AuthEnvelope) -> bool {
        !auth.signers.is_empty()
    }
}

/// Baseline object verifier for phase-1 deterministic admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicObjectVerifier;

impl ObjectVerifier for BasicObjectVerifier {
    fn verify(&self, object: &[u8], program: &Program, _spec: VmSpec) -> bool {
        !object.is_empty() && !program.code.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AdmissionError, BasicAuthVerifier, BasicObjectVerifier, ExecuteError, ExecutionContract,
        InMemoryHost, VmSpec, execute,
    };
    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
    };
    use crate::context::{
        block::BlockContext, call::CallContext, environment::EnvironmentContext,
        execution::ExecutionContext, origin::OriginContext, tx::TxContext,
    };
    use crate::receipts::outcome::ReceiptStatus;
    use crate::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};
    use crate::vm::machine::{Instruction, VmError};

    fn valid_contract(code: Vec<Instruction>) -> ExecutionContract {
        let tx = TxEnvelope::new(
            2626,
            1,
            TxKind::UserCall,
            FeeBudget::new(40, 1),
            TxPayload::new(vec![1]),
        );

        let auth = AuthEnvelope {
            domain: "tx".to_string(),
            nonce: 1,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "k1".to_string(),
                signature: vec![7_u8; 64],
            }],
        };

        let context = ExecutionContext::new(
            EnvironmentContext::new(2626, 1),
            BlockContext::new(1, 0, 0, [0_u8; 32]),
            TxContext::new([0_u8; 32], 0, 40, false, 1, 0),
            CallContext::new(0),
            OriginContext::new([0_u8; 32], [0_u8; 32], [0_u8; 32], 0),
        );

        ExecutionContract {
            tx,
            auth,
            object: vec![1, 2, 3],
            context,
            program: crate::vm::machine::Program { code },
        }
    }

    #[test]
    fn replay_is_deterministic() {
        let contract = valid_contract(vec![Instruction::Push(5), Instruction::Halt]);
        let mut host_a = InMemoryHost::default();
        let mut host_b = InMemoryHost::default();
        let spec = VmSpec::default();

        let a = execute(
            &contract,
            &mut host_a,
            spec,
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("run a");
        let b = execute(
            &contract,
            &mut host_b,
            spec,
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("run b");

        assert_eq!(a.receipt.state_root, b.receipt.state_root);
        assert_eq!(a.receipt.gas_used, b.receipt.gas_used);
    }

    #[test]
    fn rollback_on_failure_and_oom_like_oog_path() {
        let contract = valid_contract(vec![
            Instruction::Push(1),
            Instruction::Push(0),
            Instruction::Div,
        ]);
        let mut host = InMemoryHost::default();

        let out = execute(
            &contract,
            &mut host,
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("execute");

        assert_eq!(out.vm_error, Some(VmError::DivisionByZero));
        assert_eq!(out.receipt.status, ReceiptStatus::Failed);
    }

    #[test]
    fn out_of_gas_is_reported() {
        let mut contract = valid_contract(vec![
            Instruction::Push(1),
            Instruction::Push(2),
            Instruction::Add,
            Instruction::Halt,
        ]);
        contract.tx.fee_budget.gas_limit = 1;
        contract.context.tx.gas_limit = 1;

        let out = execute(
            &contract,
            &mut InMemoryHost::default(),
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("execute");

        assert_eq!(out.vm_error, Some(VmError::OutOfGas));
        assert_eq!(out.receipt.status, ReceiptStatus::Failed);
    }

    #[test]
    fn malformed_input_rejected_before_execution() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.tx.payload = TxPayload::new(vec![]);

        let err = execute(
            &contract,
            &mut InMemoryHost::default(),
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect_err("reject malformed input");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::MalformedInput));
    }

    #[test]
    fn invalid_auth_rejected_before_execution() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.auth.signers.clear();

        let err = execute(
            &contract,
            &mut InMemoryHost::default(),
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect_err("reject invalid auth");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidAuth));
    }

    #[test]
    fn invalid_object_rejected_before_execution() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.object.clear();

        let err = execute(
            &contract,
            &mut InMemoryHost::default(),
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect_err("reject invalid object");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidObject));
    }
}
