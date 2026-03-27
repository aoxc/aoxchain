use crate::context::{BlockContext, TxContext};
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;

use super::lane::LaneRegistry;
use super::pq::SignaturePolicy;
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
}

#[derive(Debug, thiserror::Error)]
pub enum HyperVmError {
    #[error("lane `{0}` not registered")]
    UnknownLane(String),
}

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
        let lane_counts = self
            .lanes
            .lane_ids()
            .into_iter()
            .map(|id| (id, 1usize))
            .collect();
        self.scheduler.plan(&lane_counts)
    }

    pub fn execute(
        &self,
        state: &mut dyn HostStateView,
        envelope: &ExecutionEnvelope,
    ) -> HyperVmResult<ExecutionReceipt> {
        let lane = self
            .lanes
            .get(envelope.lane_id.as_str())
            .ok_or_else(|| HyperVmError::UnknownLane(envelope.lane_id.clone()))?;
        lane.execute(state, &envelope.block, &envelope.tx)
            .map_err(|_| HyperVmError::UnknownLane(envelope.lane_id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AovmError;
    use crate::host::receipt::ExecutionReceipt;
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
                0,
                Vec::new(),
                Vec::new(),
            ))
        }
    }

    #[test]
    fn schedule_preview_lists_registered_lanes() {
        let mut vm = HyperVm::new(HyperVmConfig::default());
        vm.lanes_mut().register(Box::new(EchoLane));

        let decision = vm.schedule_preview();
        assert_eq!(decision.lane_order, vec!["evm"]);
    }
}
