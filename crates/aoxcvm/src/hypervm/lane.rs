use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;

/// Canonical identifier used by HyperVM to resolve an execution lane.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LaneId(pub String);

impl LaneId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Describes security and determinism characteristics of a lane implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneDescriptor {
    pub id: LaneId,
    pub deterministic: bool,
    pub max_parallelism: usize,
}

impl LaneDescriptor {
    pub fn deterministic(id: impl Into<String>) -> Self {
        Self {
            id: LaneId::new(id),
            deterministic: true,
            max_parallelism: 1,
        }
    }

    pub fn with_parallelism(mut self, max_parallelism: usize) -> Self {
        self.max_parallelism = max_parallelism.max(1);
        self
    }
}

/// Minimal execution API for pluggable HyperVM lanes.
pub trait LaneExecutor: Send + Sync {
    fn descriptor(&self) -> LaneDescriptor;

    fn execute(
        &self,
        state: &mut dyn HostStateView,
        block: &BlockContext,
        tx: &TxContext,
    ) -> Result<ExecutionReceipt, AovmError>;
}

/// In-memory lane registry with deterministic insertion order.
#[derive(Default)]
pub struct LaneRegistry {
    lanes: Vec<Box<dyn LaneExecutor>>,
}

impl LaneRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, lane: Box<dyn LaneExecutor>) {
        let incoming = lane.descriptor().id.0.clone();
        self.lanes
            .retain(|existing| existing.descriptor().id.0 != incoming);
        self.lanes.push(lane);
        self.lanes
            .sort_by(|a, b| a.descriptor().id.as_str().cmp(b.descriptor().id.as_str()));
    }

    pub fn len(&self) -> usize {
        self.lanes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lanes.is_empty()
    }

    pub fn lane_ids(&self) -> Vec<String> {
        self.lanes
            .iter()
            .map(|lane| lane.descriptor().id.as_str().to_owned())
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&dyn LaneExecutor> {
        self.lanes
            .iter()
            .find(|lane| lane.descriptor().id.as_str() == id)
            .map(std::ops::Deref::deref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{BlockContext, TxContext};
    use crate::gas::Gas;
    use crate::host::receipt::ExecutionReceipt;
    use crate::host::state::InMemoryHostState;
    use crate::vm_kind::VmKind;

    struct StubLane(&'static str);

    impl LaneExecutor for StubLane {
        fn descriptor(&self) -> LaneDescriptor {
            LaneDescriptor::deterministic(self.0)
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

    fn tx() -> TxContext {
        TxContext {
            tx_hash: [0; 32],
            sender: vec![1],
            vm_kind: VmKind::Evm,
            nonce: Some(0),
            gas_limit: 1_000 as Gas,
            max_fee_per_gas: 1,
            payload: vec![0x01],
            signature: vec![0xAB],
        }
    }

    #[test]
    fn registry_is_sorted_and_lookup_works() {
        let mut registry = LaneRegistry::new();
        registry.register(Box::new(StubLane("wasm")));
        registry.register(Box::new(StubLane("evm")));

        assert_eq!(
            registry.lane_ids(),
            vec!["evm".to_string(), "wasm".to_string()]
        );
        assert!(registry.get("evm").is_some());
        assert!(registry.get("missing").is_none());

        let mut state = InMemoryHostState::new(5_000);
        let block = BlockContext::new(1, 100, 12345, [7; 32], 1);
        let receipt = registry
            .get("evm")
            .expect("lane must exist")
            .execute(&mut state, &block, &tx())
            .expect("stub lane should execute");
        assert_eq!(receipt.gas_used, 0);
    }

    #[test]
    fn register_replaces_same_lane_id() {
        let mut registry = LaneRegistry::new();
        registry.register(Box::new(StubLane("evm")));
        registry.register(Box::new(StubLane("evm")));

        assert_eq!(registry.len(), 1);
    }
}
