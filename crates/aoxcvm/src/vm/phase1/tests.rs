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

    fn commit(&mut self, checkpoint: usize, state: crate::state::JournaledState) -> Result<(), ()> {
        self.commit_calls += 1;
        self.inner.commit(checkpoint, state)
    }
}

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
