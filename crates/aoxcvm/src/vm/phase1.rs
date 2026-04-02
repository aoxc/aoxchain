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
            Ok(ExecutionOutcome {
                receipt: envelope.result.receipt,
                stack: envelope.result.stack,
                gas_used: envelope.result.receipt.gas_used,
                halt_reason: HaltReason::VmError(err.clone()),
                spec_version: spec.spec_version,
                journal_committed: false,
                vm_error: Some(err),
            })
        }
        None => {
            host.commit(checkpoint, envelope.result.final_state)
                .map_err(|_| ExecuteError::HostCommit)?;
            Ok(ExecutionOutcome {
                receipt: envelope.result.receipt,
                stack: envelope.result.stack,
                gas_used: envelope.result.receipt.gas_used,
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

#[cfg(test)]
mod tests {
    use super::{
        AdmissionError, AuthVerifier, BasicAuthVerifier, BasicObjectVerifier, ExecuteError,
        ExecutionContract, HaltReason, Host, InMemoryHost, ObjectVerifier, VmSpec, execute,
    };
    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
    };
    use crate::context::{
        block::BlockContext,
        call::CallContext,
        environment::EnvironmentContext,
        execution::{ContextError, ExecutionContext},
        origin::OriginContext,
        tx::TxContext,
    };
    use crate::receipts::outcome::ReceiptStatus;
    use crate::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};
    use crate::vm::machine::{Instruction, Program, VmError};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Host spy used to validate lifecycle ordering and side effects at the
    /// host boundary.
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct SpyHost {
        inner: InMemoryHost,
        checkpoint_calls: usize,
        rollback_calls: usize,
        commit_calls: usize,
    }

    impl Host for SpyHost {
        type Error = ();

        fn load_state(&self) -> crate::state::JournaledState {
            self.inner.load_state()
        }

        fn checkpoint(&mut self) -> Result<usize, ()> {
            self.checkpoint_calls += 1;
            self.inner.checkpoint()
        }

        fn rollback(&mut self, checkpoint: usize) -> Result<(), ()> {
            self.rollback_calls += 1;
            self.inner.rollback(checkpoint)
        }

        fn commit(
            &mut self,
            checkpoint: usize,
            state: crate::state::JournaledState,
        ) -> Result<(), ()> {
            self.commit_calls += 1;
            self.inner.commit(checkpoint, state)
        }
    }

    /// Counting verifier used to prove admission short-circuit ordering.
    #[derive(Debug, Default)]
    struct CountingAuthVerifier {
        calls: AtomicUsize,
        allow: bool,
    }

    impl CountingAuthVerifier {
        fn allowing() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                allow: true,
            }
        }

        fn rejecting() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                allow: false,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl AuthVerifier for CountingAuthVerifier {
        fn verify(&self, _tx: &TxEnvelope, _auth: &AuthEnvelope) -> bool {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.allow
        }
    }

    /// Counting verifier used to confirm that object verification is reached
    /// only after successful auth admission.
    #[derive(Debug, Default)]
    struct CountingObjectVerifier {
        calls: AtomicUsize,
        allow: bool,
    }

    impl CountingObjectVerifier {
        fn allowing() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                allow: true,
            }
        }

        fn rejecting() -> Self {
            Self {
                calls: AtomicUsize::new(0),
                allow: false,
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl ObjectVerifier for CountingObjectVerifier {
        fn verify(&self, _object: &[u8], _program: &Program, _spec: VmSpec) -> bool {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.allow
        }
    }

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
            program: Program { code },
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
        assert_eq!(a.stack, b.stack);
        assert_eq!(a.halt_reason, b.halt_reason);
        assert_eq!(a.vm_error, b.vm_error);
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
        assert_eq!(
            out.halt_reason,
            HaltReason::VmError(VmError::DivisionByZero)
        );
        assert!(!out.journal_committed);
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
        assert_eq!(out.gas_used, out.receipt.gas_used);
        assert!(!out.journal_committed);
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
        let contract = valid_contract(vec![Instruction::Halt]);
        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::rejecting();
        let object = CountingObjectVerifier::allowing();

        let err = execute(&contract, &mut host, VmSpec::default(), &auth, &object)
            .expect_err("reject invalid auth");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidAuth));
        assert_eq!(auth.calls(), 1);
        assert_eq!(object.calls(), 0);
        assert_eq!(host.checkpoint_calls, 0);
        assert_eq!(host.rollback_calls, 0);
        assert_eq!(host.commit_calls, 0);
    }

    #[test]
    fn invalid_object_rejected_before_execution() {
        let contract = valid_contract(vec![Instruction::Halt]);
        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::allowing();
        let object = CountingObjectVerifier::rejecting();

        let err = execute(&contract, &mut host, VmSpec::default(), &auth, &object)
            .expect_err("reject invalid object");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidObject));
        assert_eq!(auth.calls(), 1);
        assert_eq!(object.calls(), 1);
        assert_eq!(host.checkpoint_calls, 0);
        assert_eq!(host.rollback_calls, 0);
        assert_eq!(host.commit_calls, 0);
    }

    #[test]
    fn oversize_object_is_fail_closed_before_object_verifier() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.object = vec![1, 2, 3];

        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::allowing();
        let object = CountingObjectVerifier::allowing();
        let spec = VmSpec {
            max_object_bytes: 2,
            ..VmSpec::default()
        };

        let err = execute(&contract, &mut host, spec, &auth, &object).expect_err("reject object");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidObject));
        assert_eq!(auth.calls(), 1);
        assert_eq!(object.calls(), 0);
        assert_eq!(host.checkpoint_calls, 0);
        assert_eq!(host.rollback_calls, 0);
        assert_eq!(host.commit_calls, 0);
    }

    #[test]
    fn execution_failure_rolls_back_and_does_not_commit() {
        let contract = valid_contract(vec![
            Instruction::Push(1),
            Instruction::Push(0),
            Instruction::Div,
        ]);
        let mut host = SpyHost::default();

        let out = execute(
            &contract,
            &mut host,
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("execute");

        assert_eq!(out.vm_error, Some(VmError::DivisionByZero));
        assert!(!out.journal_committed);
        assert_eq!(host.checkpoint_calls, 1);
        assert_eq!(host.rollback_calls, 1);
        assert_eq!(host.commit_calls, 0);
    }

    #[test]
    fn successful_execution_commits_without_rollback() {
        let contract = valid_contract(vec![Instruction::Push(7), Instruction::Halt]);
        let mut host = SpyHost::default();

        let out = execute(
            &contract,
            &mut host,
            VmSpec::default(),
            &BasicAuthVerifier,
            &BasicObjectVerifier,
        )
        .expect("execute");

        assert_eq!(out.vm_error, None);
        assert_eq!(out.halt_reason, HaltReason::Success);
        assert!(out.journal_committed);
        assert_eq!(out.spec_version, VmSpec::default().spec_version);
        assert_eq!(out.receipt.status, ReceiptStatus::Success);
        assert_eq!(host.checkpoint_calls, 1);
        assert_eq!(host.rollback_calls, 0);
        assert_eq!(host.commit_calls, 1);
    }

    #[test]
    fn malformed_input_takes_priority_over_auth_and_object_failures() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.tx.payload = TxPayload::new(vec![]);

        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::rejecting();
        let object = CountingObjectVerifier::rejecting();

        let err = execute(&contract, &mut host, VmSpec::default(), &auth, &object)
            .expect_err("reject malformed input first");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::MalformedInput));
        assert_eq!(auth.calls(), 0);
        assert_eq!(object.calls(), 0);
        assert_eq!(host.checkpoint_calls, 0);
    }

    #[test]
    fn invalid_context_takes_priority_over_auth_failure() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.context.tx.gas_limit = 0;

        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::rejecting();
        let object = CountingObjectVerifier::rejecting();

        let err = execute(&contract, &mut host, VmSpec::default(), &auth, &object)
            .expect_err("reject context before auth");

        assert!(matches!(
            err,
            ExecuteError::Admission(AdmissionError::Context(ContextError::ZeroGasLimit))
        ));
        assert_eq!(auth.calls(), 0);
        assert_eq!(object.calls(), 0);
        assert_eq!(host.checkpoint_calls, 0);
    }

    #[test]
    fn invalid_auth_takes_priority_over_oversize_object() {
        let mut contract = valid_contract(vec![Instruction::Halt]);
        contract.object = vec![1, 2, 3];

        let mut host = SpyHost::default();
        let auth = CountingAuthVerifier::rejecting();
        let object = CountingObjectVerifier::allowing();
        let spec = VmSpec {
            max_object_bytes: 1,
            ..VmSpec::default()
        };

        let err = execute(&contract, &mut host, spec, &auth, &object)
            .expect_err("reject auth before object size check");

        assert_eq!(err, ExecuteError::Admission(AdmissionError::InvalidAuth));
        assert_eq!(auth.calls(), 1);
        assert_eq!(object.calls(), 0);
        assert_eq!(host.checkpoint_calls, 0);
    }
}
