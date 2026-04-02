//! Canonical phase-1 execution entrypoint and contracts.

use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{ContractDescriptor, VmTarget};

use crate::auth::{
    envelope::{AuthEnvelope, AuthEnvelopeLimits},
    scheme::AuthProfile,
};
use crate::context::{
    deterministic::DeterminismLimits,
    execution::{ContextError, ExecutionContext},
};
use crate::errors::AoxcvmError;
use crate::receipts::outcome::ExecutionReceipt;
use crate::state::JournaledState;
use crate::tx::envelope::TxEnvelope;
use crate::vm::machine::{Machine, Program, VmError};

pub type Receipt = ExecutionReceipt;

/// Canonical phase-1 transaction input to the single public kernel entrypoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phase1Tx {
    pub tx: TxEnvelope,
    pub auth: AuthEnvelope,
    pub context: ExecutionContext,
    pub object: Vec<u8>,
    pub entrypoint: String,
    pub program: Program,
}

/// Canonical execution spec resolved from configuration + descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmSpec {
    pub gas_limit: u64,
    pub max_memory: usize,
    pub max_object_bytes: usize,
    pub max_call_depth: u16,
    pub min_spec_version: u32,
    pub vm_target: VmTarget,
    pub strict_mode: bool,
}

impl VmSpec {
    pub fn from_config(
        config: &ContractsConfig,
        descriptor: &ContractDescriptor,
    ) -> Result<Self, SpecError> {
        if !config
            .artifact_policy
            .allowed_vm_targets
            .contains(&descriptor.manifest.vm_target)
        {
            return Err(SpecError::VmTargetDisabledByConfig);
        }

        Ok(Self {
            gas_limit: 1_000_000,
            max_memory: 1024 * 1024,
            max_object_bytes: config.artifact_policy.max_artifact_size as usize,
            max_call_depth: 64,
            min_spec_version: 1,
            vm_target: descriptor.manifest.vm_target.clone(),
            strict_mode: true,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    VmTargetDisabledByConfig,
}

/// Canonical taxonomy upper crates can classify without parsing strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultClass {
    Admission,
    Auth,
    Verifier,
    Execution,
    Memory,
    Gas,
    State,
    Host,
    FatalKernel,
}

/// Canonical phase-1 outcome consumed by upper layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    pub receipt: Receipt,
    pub stack: Vec<u64>,
    pub vm_error: Option<VmError>,
    pub fault_class: Option<FaultClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    MalformedInput,
    InvalidDescriptor,
    VmTargetMismatch,
    EntrypointNotFound,
    InvalidObject,
    Context(ContextError),
}

impl From<ContextError> for AdmissionError {
    fn from(value: ContextError) -> Self {
        Self::Context(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    Spec(SpecError),
    Admission(AdmissionError),
    Auth(AoxcvmError),
    Host,
}

impl From<SpecError> for ExecuteError {
    fn from(value: SpecError) -> Self {
        Self::Spec(value)
    }
}

/// Host boundary: state mutation visibility flows only through checkpoint/rollback/commit.
pub trait Host {
    fn load_state(&self) -> JournaledState;
    fn checkpoint(&mut self) -> Result<usize, ()>;
    fn rollback(&mut self, checkpoint: usize) -> Result<(), ()>;
    fn commit(&mut self, checkpoint: usize, state: JournaledState) -> Result<(), ()>;
}

pub trait ObjectVerifier {
    fn verify(
        &self,
        descriptor: &ContractDescriptor,
        object: &[u8],
        entrypoint: &str,
        program: &Program,
        spec: &VmSpec,
    ) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanonicalObjectVerifier;

impl ObjectVerifier for CanonicalObjectVerifier {
    fn verify(
        &self,
        descriptor: &ContractDescriptor,
        object: &[u8],
        entrypoint: &str,
        program: &Program,
        spec: &VmSpec,
    ) -> bool {
        if object.is_empty() || program.code.is_empty() || object.len() > spec.max_object_bytes {
            return false;
        }

        descriptor
            .manifest
            .entrypoints
            .iter()
            .any(|ep| ep.name == entrypoint)
    }
}

/// Single canonical phase-1 entrypoint.
///
/// Ordering is fixed and fail-closed:
/// input admission -> auth verify -> object/descriptor verify -> execute.
pub fn execute(
    tx: &Phase1Tx,
    descriptor: &ContractDescriptor,
    host: &mut impl Host,
    spec: &VmSpec,
) -> Result<ExecutionOutcome, ExecuteError> {
    execute_with(tx, descriptor, host, spec, &CanonicalObjectVerifier)
}

fn execute_with(
    tx: &Phase1Tx,
    descriptor: &ContractDescriptor,
    host: &mut impl Host,
    spec: &VmSpec,
    object_verifier: &impl ObjectVerifier,
) -> Result<ExecutionOutcome, ExecuteError> {
    if tx.tx.payload.is_empty() || tx.entrypoint.trim().is_empty() {
        return Err(ExecuteError::Admission(AdmissionError::MalformedInput));
    }

    if descriptor.manifest.entrypoints.is_empty() {
        return Err(ExecuteError::Admission(AdmissionError::InvalidDescriptor));
    }

    if descriptor.manifest.vm_target != spec.vm_target {
        return Err(ExecuteError::Admission(AdmissionError::VmTargetMismatch));
    }

    tx.context
        .validate(DeterminismLimits {
            max_call_depth: spec.max_call_depth,
            max_gas_limit: spec.gas_limit,
            min_spec_version: spec.min_spec_version,
        })
        .map_err(AdmissionError::from)
        .map_err(ExecuteError::Admission)?;

    tx.auth
        .validate(AuthProfile::Legacy, AuthEnvelopeLimits::default())
        .map_err(ExecuteError::Auth)?;

    if !object_verifier.verify(descriptor, &tx.object, &tx.entrypoint, &tx.program, spec) {
        return Err(ExecuteError::Admission(AdmissionError::InvalidObject));
    }

    if !descriptor
        .manifest
        .entrypoints
        .iter()
        .any(|ep| ep.name == tx.entrypoint)
    {
        return Err(ExecuteError::Admission(AdmissionError::EntrypointNotFound));
    }

    let checkpoint = host.checkpoint().map_err(|_| ExecuteError::Host)?;
    let initial_state = host.load_state();

    let envelope = Machine::with_state(
        tx.program.clone(),
        tx.tx.fee_budget.gas_limit,
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
                fault_class: Some(classify_vm_error(&err)),
                vm_error: Some(err),
            })
        }
        None => {
            host.commit(checkpoint, envelope.result.final_state)
                .map_err(|_| ExecuteError::Host)?;
            Ok(ExecutionOutcome {
                receipt: envelope.result.receipt,
                stack: envelope.result.stack,
                fault_class: None,
                vm_error: None,
            })
        }
    }
}

fn classify_vm_error(err: &VmError) -> FaultClass {
    match err {
        VmError::OutOfGas => FaultClass::Gas,
        VmError::MemoryOutOfBounds => FaultClass::Memory,
        VmError::InvalidCheckpoint => FaultClass::State,
        VmError::StackUnderflow | VmError::DivisionByZero | VmError::InvalidProgramCounter => {
            FaultClass::Execution
        }
    }
}

/// Minimal in-memory host implementation for deterministic tests and adapters.
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

#[cfg(test)]
mod tests {
    use aoxcontract::{
        ArtifactDigest, ArtifactDigestAlgorithm, ContractDescriptor, ContractMetadata, Entrypoint,
        VmTarget,
    };

    use super::{
        AdmissionError, ExecuteError, FaultClass, InMemoryHost, Phase1Tx, VmSpec, execute,
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
    use crate::vm::machine::{Instruction, Program, VmError};

    fn descriptor(vm_target: VmTarget) -> ContractDescriptor {
        let manifest = aoxcsdk::contracts::builder::ContractManifestBuilder::new()
            .with_name("phase1")
            .with_package("aox.phase1")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(vm_target)
            .with_artifact_digest(ArtifactDigest {
                algorithm: ArtifactDigestAlgorithm::Sha256,
                value: "5f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222".into(),
            })
            .with_artifact_location("ipfs://phase1/module")
            .with_metadata(ContractMetadata {
                display_name: "Phase1 Contract".into(),
                description: Some("phase1 test".into()),
                author: Some("AOX".into()),
                organization: Some("AOX".into()),
                source_reference: None,
                tags: vec!["phase1".into()],
                created_at: None,
                updated_at: None,
                audit_reference: Some("approved".into()),
                notes: None,
            })
            .add_entrypoint(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap())
            .build()
            .unwrap();

        ContractDescriptor::new(manifest).unwrap()
    }

    fn tx(code: Vec<Instruction>) -> Phase1Tx {
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

        Phase1Tx {
            tx,
            auth,
            context,
            object: vec![1, 2, 3],
            entrypoint: "execute".to_string(),
            program: Program { code },
        }
    }

    #[test]
    fn config_descriptor_vm_triangle_is_fail_closed() {
        let mut config = aoxconfig::contracts::ContractsConfig::default();
        config.artifact_policy.allowed_vm_targets = vec![VmTarget::Evm];
        let err = VmSpec::from_config(&config, &descriptor(VmTarget::Wasm)).expect_err("closed");
        assert_eq!(err, super::SpecError::VmTargetDisabledByConfig);
    }

    #[test]
    fn replay_is_deterministic_via_public_execute() {
        let desc = descriptor(VmTarget::Wasm);
        let spec = VmSpec::from_config(&aoxconfig::contracts::ContractsConfig::default(), &desc)
            .expect("spec");
        let payload = tx(vec![Instruction::Push(5), Instruction::Halt]);

        let a = execute(&payload, &desc, &mut InMemoryHost::default(), &spec).expect("a");
        let b = execute(&payload, &desc, &mut InMemoryHost::default(), &spec).expect("b");

        assert_eq!(a.receipt.state_root, b.receipt.state_root);
        assert_eq!(a.receipt.gas_used, b.receipt.gas_used);
    }

    #[test]
    fn out_of_gas_maps_to_gas_fault_class() {
        let desc = descriptor(VmTarget::Wasm);
        let spec = VmSpec::from_config(&aoxconfig::contracts::ContractsConfig::default(), &desc)
            .expect("spec");
        let mut payload = tx(vec![
            Instruction::Push(1),
            Instruction::Push(2),
            Instruction::Add,
            Instruction::Halt,
        ]);
        payload.tx.fee_budget.gas_limit = 1;
        payload.context.tx.gas_limit = 1;

        let out = execute(&payload, &desc, &mut InMemoryHost::default(), &spec).expect("out");
        assert_eq!(out.vm_error, Some(VmError::OutOfGas));
        assert_eq!(out.fault_class, Some(FaultClass::Gas));
        assert_eq!(out.receipt.status, ReceiptStatus::Failed);
    }

    #[test]
    fn malformed_input_rejected_before_execution() {
        let desc = descriptor(VmTarget::Wasm);
        let spec = VmSpec::from_config(&aoxconfig::contracts::ContractsConfig::default(), &desc)
            .expect("spec");
        let mut payload = tx(vec![Instruction::Halt]);
        payload.tx.payload = TxPayload::new(vec![]);

        let err = execute(&payload, &desc, &mut InMemoryHost::default(), &spec).expect_err("err");
        assert_eq!(err, ExecuteError::Admission(AdmissionError::MalformedInput));
    }

    #[test]
    fn invalid_auth_rejected_before_execution() {
        let desc = descriptor(VmTarget::Wasm);
        let spec = VmSpec::from_config(&aoxconfig::contracts::ContractsConfig::default(), &desc)
            .expect("spec");
        let mut payload = tx(vec![Instruction::Halt]);
        payload.auth.signers.clear();

        let err = execute(&payload, &desc, &mut InMemoryHost::default(), &spec).expect_err("err");
        assert!(matches!(err, ExecuteError::Auth(_)));
    }
}
