use std::collections::BTreeMap;

use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;

use super::lane::LaneRegistry;
use super::pq::{HybridSignature, SignaturePolicy};
use super::scheduler::{DeterministicScheduler, SchedulingDecision};

#[derive(Debug, Clone)]
pub struct HyperVmConfig {
    pub signature_policy: SignaturePolicy,
    pub max_scheduler_partitions: usize,
}

impl Default for HyperVmConfig {
    fn default() -> Self {
        Self {
            signature_policy: SignaturePolicy::Hybrid,
            max_scheduler_partitions: 4,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionEnvelope {
    pub lane_id: String,
    pub block: BlockContext,
    pub tx: TxContext,
    pub post_quantum_signature: Option<Vec<u8>>,
}

impl ExecutionEnvelope {
    pub fn hybrid_signature(&self) -> HybridSignature {
        HybridSignature {
            classical: self.tx.signature.clone(),
            post_quantum: self.post_quantum_signature.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HyperVmError {
    UnknownLane(String),
    InvalidEnvelope(&'static str),
    SignaturePolicyViolation {
        lane: String,
        policy: SignaturePolicy,
    },
    LaneExecutionFailed {
        lane: String,
        source: AovmError,
    },
}

impl core::fmt::Display for HyperVmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownLane(lane) => write!(f, "lane `{lane}` not registered"),
            Self::InvalidEnvelope(msg) => write!(f, "invalid envelope: {msg}"),
            Self::SignaturePolicyViolation { lane, policy } => write!(
                f,
                "signature policy violation for lane `{lane}` under policy `{policy:?}`"
            ),
            Self::LaneExecutionFailed { lane, source } => {
                write!(f, "lane `{lane}` failed: {source}")
            }
        }
    }
}

impl std::error::Error for HyperVmError {}

pub type HyperVmResult<T> = Result<T, HyperVmError>;

pub struct HyperVm {
    config: HyperVmConfig,
    scheduler: DeterministicScheduler,
    lanes: LaneRegistry,
}

impl HyperVm {
    pub fn new(config: HyperVmConfig) -> Self {
        let scheduler = DeterministicScheduler::new(config.max_scheduler_partitions);
        Self {
            config,
            scheduler,
            lanes: LaneRegistry::new(),
        }
    }

    pub fn config(&self) -> &HyperVmConfig {
        &self.config
    }

    pub fn scheduler(&self) -> &DeterministicScheduler {
        &self.scheduler
    }

    pub fn lanes(&self) -> &LaneRegistry {
        &self.lanes
    }

    pub fn lanes_mut(&mut self) -> &mut LaneRegistry {
        &mut self.lanes
    }

    pub fn schedule_preview(&self) -> SchedulingDecision {
        let lane_counts: BTreeMap<String, usize> = self
            .lanes
            .lane_ids()
            .into_iter()
            .map(|id| (id, 1usize))
            .collect();
        self.scheduler.plan(&lane_counts)
    }

    pub fn plan_batch(&self, envelopes: &[ExecutionEnvelope]) -> SchedulingDecision {
        let mut lane_counts = BTreeMap::new();
        for envelope in envelopes {
            *lane_counts
                .entry(envelope.lane_id.clone())
                .or_insert(0usize) += 1;
        }
        self.scheduler.plan(&lane_counts)
    }

    pub fn execute(
        &self,
        state: &mut dyn HostStateView,
        envelope: &ExecutionEnvelope,
    ) -> HyperVmResult<ExecutionReceipt> {
        self.validate_envelope(envelope)?;
        self.validate_signature_policy(envelope)?;

        let lane = self
            .lanes
            .get(envelope.lane_id.as_str())
            .ok_or_else(|| HyperVmError::UnknownLane(envelope.lane_id.clone()))?;
        lane.execute(state, &envelope.block, &envelope.tx)
            .map_err(|source| HyperVmError::LaneExecutionFailed {
                lane: envelope.lane_id.clone(),
                source,
            })
    }

    pub fn execute_batch(
        &self,
        state: &mut dyn HostStateView,
        envelopes: &[ExecutionEnvelope],
    ) -> HyperVmResult<Vec<ExecutionReceipt>> {
        let plan = self.plan_batch(envelopes);
        let mut out = Vec::with_capacity(envelopes.len());

        for lane_id in plan.lane_order {
            for envelope in envelopes.iter().filter(|e| e.lane_id == lane_id) {
                out.push(self.execute(state, envelope)?);
            }
        }

        Ok(out)
    }

    fn validate_envelope(&self, envelope: &ExecutionEnvelope) -> HyperVmResult<()> {
        if envelope.lane_id.trim().is_empty() {
            return Err(HyperVmError::InvalidEnvelope("lane id is empty"));
        }
        envelope
            .tx
            .validate_basic()
            .map_err(|_| HyperVmError::InvalidEnvelope("tx basic validation failed"))?;
        Ok(())
    }

    fn validate_signature_policy(&self, envelope: &ExecutionEnvelope) -> HyperVmResult<()> {
        let signature = envelope.hybrid_signature();
        if !signature.is_valid_for_policy(self.config.signature_policy) {
            return Err(HyperVmError::SignaturePolicyViolation {
                lane: envelope.lane_id.clone(),
                policy: self.config.signature_policy,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gas::Gas;
    use crate::host::state::InMemoryHostState;
    use crate::vm_kind::VmKind;

    struct EchoLane;

    impl crate::hypervm::lane::LaneExecutor for EchoLane {
        fn descriptor(&self) -> crate::hypervm::lane::LaneDescriptor {
            crate::hypervm::lane::LaneDescriptor::deterministic("evm")
        }

        fn execute(
            &self,
            _state: &mut dyn HostStateView,
            _block: &BlockContext,
            _tx: &TxContext,
        ) -> Result<ExecutionReceipt, AovmError> {
            Ok(ExecutionReceipt::success(
                VmKind::Evm,
                21000,
                Vec::new(),
                b"ok".to_vec(),
            ))
        }
    }

    fn envelope() -> ExecutionEnvelope {
        ExecutionEnvelope {
            lane_id: "evm".to_string(),
            block: BlockContext::new(1, 100, 12345, [7; 32], 1),
            tx: TxContext {
                tx_hash: [1; 32],
                sender: vec![0xAA],
                vm_kind: VmKind::Evm,
                nonce: Some(0),
                gas_limit: 100_000 as Gas,
                max_fee_per_gas: 10,
                payload: vec![0x01],
                signature: vec![0xBB],
            },
            post_quantum_signature: Some(vec![0xCC]),
        }
    }

    #[test]
    fn schedule_preview_lists_registered_lanes() {
        let mut vm = HyperVm::new(HyperVmConfig::default());
        vm.lanes_mut().register(Box::new(EchoLane));

        let decision = vm.schedule_preview();
        assert_eq!(decision.lane_order, vec!["evm"]);
    }

    #[test]
    fn execute_batch_runs_in_lane_order() {
        let mut vm = HyperVm::new(HyperVmConfig::default());
        vm.lanes_mut().register(Box::new(EchoLane));

        let mut state = InMemoryHostState::new(1_000_000);
        let receipts = vm
            .execute_batch(&mut state, &[envelope(), envelope()])
            .expect("batch execute should succeed");

        assert_eq!(receipts.len(), 2);
        assert!(receipts.iter().all(|receipt| receipt.success));
    }

    #[test]
    fn signature_policy_violation_is_rejected() {
        let mut vm = HyperVm::new(HyperVmConfig {
            signature_policy: SignaturePolicy::PostQuantumOnly,
            max_scheduler_partitions: 2,
        });
        vm.lanes_mut().register(Box::new(EchoLane));

        let mut invalid = envelope();
        invalid.post_quantum_signature = None;

        let mut state = InMemoryHostState::new(1_000_000);
        let err = vm
            .execute(&mut state, &invalid)
            .expect_err("should fail policy");

        assert!(matches!(err, HyperVmError::SignaturePolicyViolation { .. }));
    }
}
