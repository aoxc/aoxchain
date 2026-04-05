//! Canonical phase-1 execution entrypoint and contracts.

use crate::auth::envelope::AuthEnvelope;
use crate::context::execution::{ContextError, ExecutionContext};
use crate::receipts::outcome::ExecutionReceipt;
use crate::state::JournaledState;
use crate::tx::envelope::TxEnvelope;
use crate::vm::machine::{Machine, Program, VmError};
use aoxconfig::contracts::ContractsConfig;
use aoxcontract::ContractDescriptor;

/// Stable execution context contract.
pub type ExecutionContractContext = ExecutionContext;

/// Stable receipt contract.
pub type Receipt = ExecutionReceipt;

/// Canonical VM spec used by the phase-1 kernel entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VmSpec {
    pub spec_version: u16,
    pub gas_limit: u64,
    pub max_memory: usize,
    pub max_object_bytes: usize,
    pub strict_mode: bool,
}

/// Configuration-time spec derivation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecError {
    VmTargetDisabledByConfig,
}

impl VmSpec {
    /// Build a phase-1 VM spec from contract-policy config with fail-closed
    /// VM target admission.
    pub fn from_config(
        config: &ContractsConfig,
        descriptor: &ContractDescriptor,
    ) -> Result<Self, SpecError> {
        let target_enabled = config
            .artifact_policy
            .allowed_vm_targets
            .iter()
            .any(|target| target == &descriptor.manifest.vm_target);

        if !target_enabled {
            return Err(SpecError::VmTargetDisabledByConfig);
        }

        Ok(Self {
            max_object_bytes: config.artifact_policy.max_artifact_size as usize,
            strict_mode: config.artifact_policy.review_required,
            ..Self::default()
        })
    }
}

impl Default for VmSpec {
    fn default() -> Self {
        Self {
            spec_version: 1,
            gas_limit: 1_000_000,
            max_memory: 1024 * 1024,
            max_object_bytes: 64 * 1024,
            strict_mode: true,
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
///
/// This structure intentionally exposes only the stable, protocol-relevant
/// outcome surface required by Phase 1 integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub receipt: Receipt,
    pub stack: Vec<u64>,
    pub gas_used: u64,
    pub halt_reason: HaltReason,
    pub spec_version: u16,
    pub journal_committed: bool,
    pub vm_error: Option<VmError>,
}

/// Canonical halt classification exposed by the phase-1 kernel output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HaltReason {
    Success,
    VmError(VmError),
}

/// Pre-execution admission errors.
///
/// These errors are raised strictly before execution begins. They must never
/// represent an in-VM runtime failure.
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
    HostCheckpoint,
    HostRollback,
    HostCommit,
}

impl From<ContextError> for ExecuteError {
    fn from(value: ContextError) -> Self {
        Self::Admission(AdmissionError::Context(value))
    }
}

/// Host boundary: the VM kernel must never persist state directly.
///
/// All durable state transitions must flow through this interface so that
/// checkpoint, rollback, and commit semantics remain explicit and testable.
pub trait Host {
    type Error;

    fn load_state(&self) -> JournaledState;
    fn checkpoint(&mut self) -> Result<usize, Self::Error>;
    fn rollback(&mut self, checkpoint: usize) -> Result<(), Self::Error>;
    fn commit(&mut self, checkpoint: usize, state: JournaledState) -> Result<(), Self::Error>;
}

/// Authentication admission boundary.
pub trait AuthVerifier {
    fn verify(&self, tx: &TxEnvelope, auth: &AuthEnvelope) -> bool;
}

/// Object and bytecode admission boundary.
pub trait ObjectVerifier {
    fn verify(&self, object: &[u8], program: &Program, spec: VmSpec) -> bool;
}

/// Canonical phase-1 kernel entry.
///
/// The lifecycle ordering is intentionally strict:
/// 1. malformed-input rejection,
/// 2. deterministic context validation,
/// 3. authentication verification,
/// 4. object verification,
/// 5. host checkpoint,
/// 6. VM execution,
/// 7. rollback or commit.
///
/// This ordering is a security boundary. Invalid admission input must fail
/// closed before any state-transition lifecycle begins.
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

    let checkpoint = host
        .checkpoint()
        .map_err(|_| ExecuteError::HostCheckpoint)?;
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
            host.rollback(checkpoint)
                .map_err(|_| ExecuteError::HostRollback)?;

            let receipt = envelope.result.receipt;
            let gas_used = receipt.gas_used;
            let stack = envelope.result.stack;

            Ok(ExecutionOutcome {
                receipt,
                stack,
                gas_used,
                halt_reason: HaltReason::VmError(err.clone()),
                spec_version: spec.spec_version,
                journal_committed: false,
                vm_error: Some(err),
            })
        }
        None => {
            host.commit(checkpoint, envelope.result.final_state)
                .map_err(|_| ExecuteError::HostCommit)?;

            let receipt = envelope.result.receipt;
            let gas_used = receipt.gas_used;
            let stack = envelope.result.stack;

            Ok(ExecutionOutcome {
                receipt,
                stack,
                gas_used,
                halt_reason: HaltReason::Success,
                spec_version: spec.spec_version,
                journal_committed: true,
                vm_error: None,
            })
        }
    }
}

/// Minimal in-memory host implementation for deterministic tests.
///
/// This host intentionally keeps semantics small and explicit. It is suitable
/// for unit and lifecycle tests, but not intended as a production storage
/// backend.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InMemoryHost {
    state: JournaledState,
    checkpoints: Vec<JournaledState>,
}

impl Host for InMemoryHost {
    type Error = ();

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

/// Baseline verifier that enforces minimum auth envelope shape.
///
/// Phase 1 intentionally keeps this implementation conservative.
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
