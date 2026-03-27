use std::collections::{BTreeMap, BTreeSet};

use crate::gas::Gas;

pub type StateKey = Vec<u8>;
pub type StateValue = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTxEnvelope {
    pub tx_id: [u8; 32],
    pub sender: Vec<u8>,
    pub nonce: u64,
    pub lane: LaneId,
    pub gas_limit: Gas,
    pub max_fee_per_gas: u128,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
}

impl CanonicalTxEnvelope {
    pub fn validate(&self) -> Result<(), KernelError> {
        if self.sender.is_empty() {
            return Err(KernelError::InvalidTransaction("sender must not be empty"));
        }
        if self.payload.is_empty() {
            return Err(KernelError::InvalidTransaction("payload must not be empty"));
        }
        if self.gas_limit == 0 {
            return Err(KernelError::InvalidTransaction(
                "gas_limit must be positive",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockExecutionContext {
    pub chain_id: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub proposer: [u8; 32],
    pub base_fee: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LaneId {
    Core,
    Evm,
    Wasm,
    Other(u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub lane: LaneId,
    pub topic: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub tx_id: [u8; 32],
    pub lane: LaneId,
    pub success: bool,
    pub gas_used: Gas,
    pub events: Vec<Event>,
    pub output: Vec<u8>,
    pub error: Option<KernelError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    InvalidTransaction(&'static str),
    LaneNotRegistered(LaneId),
    GasExhausted,
    StateViolation(&'static str),
    DeterministicAbort { code: u16, message: &'static str },
}

pub trait HostState {
    fn get(&self, key: &[u8]) -> Option<StateValue>;
    fn set(&mut self, key: StateKey, value: StateValue);
    fn delete(&mut self, key: &[u8]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalOp {
    Put { key: StateKey, value: StateValue },
    Delete { key: StateKey },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateJournal {
    ops: Vec<JournalOp>,
}

impl StateJournal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&mut self, key: StateKey, value: StateValue) {
        self.ops.push(JournalOp::Put { key, value });
    }

    pub fn delete(&mut self, key: StateKey) {
        self.ops.push(JournalOp::Delete { key });
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn commit(self, state: &mut impl HostState) {
        for op in self.ops {
            match op {
                JournalOp::Put { key, value } => state.set(key, value),
                JournalOp::Delete { key } => state.delete(&key),
            }
        }
    }

    pub fn revert(self) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelMeter {
    limit: Gas,
    used: Gas,
}

impl FuelMeter {
    pub fn new(limit: Gas) -> Self {
        Self { limit, used: 0 }
    }

    pub fn charge(&mut self, amount: Gas) -> Result<(), KernelError> {
        let next = self
            .used
            .checked_add(amount)
            .ok_or(KernelError::GasExhausted)?;
        if next > self.limit {
            return Err(KernelError::GasExhausted);
        }
        self.used = next;
        Ok(())
    }

    pub fn used(&self) -> Gas {
        self.used
    }

    pub fn remaining(&self) -> Gas {
        self.limit.saturating_sub(self.used)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuelSchedule {
    pub tx_base: Gas,
    pub byte_cost: Gas,
    pub event_base: Gas,
    pub state_write_cost: Gas,
    pub state_delete_cost: Gas,
}

impl Default for FuelSchedule {
    fn default() -> Self {
        Self {
            tx_base: 21_000,
            byte_cost: 4,
            event_base: 375,
            state_write_cost: 2_000,
            state_delete_cost: 500,
        }
    }
}

pub struct ExecutionEnv<'a, S: HostState> {
    pub block: &'a BlockExecutionContext,
    pub tx: &'a CanonicalTxEnvelope,
    pub state: &'a mut S,
    pub journal: &'a mut StateJournal,
    pub fuel: &'a mut FuelMeter,
    pub schedule: &'a FuelSchedule,
}

impl<'a, S: HostState> ExecutionEnv<'a, S> {
    pub fn read_state(&mut self, key: &[u8]) -> Option<StateValue> {
        self.state.get(key)
    }

    pub fn write_state(&mut self, key: StateKey, value: StateValue) -> Result<(), KernelError> {
        self.fuel.charge(self.schedule.state_write_cost)?;
        self.journal.put(key, value);
        Ok(())
    }

    pub fn delete_state(&mut self, key: StateKey) -> Result<(), KernelError> {
        self.fuel.charge(self.schedule.state_delete_cost)?;
        self.journal.delete(key);
        Ok(())
    }

    pub fn emit_event(&mut self, topic: Vec<u8>, data: Vec<u8>) -> Result<Event, KernelError> {
        self.fuel.charge(self.schedule.event_base)?;
        Ok(Event {
            lane: self.tx.lane,
            topic,
            data,
        })
    }
}

pub struct LaneOutput {
    pub output: Vec<u8>,
    pub events: Vec<Event>,
}

pub trait LaneAdapter<S: HostState> {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CoreCall {
    Noop,
    Set { key: StateKey, value: StateValue },
    Delete { key: StateKey },
}

fn decode_core_call(payload: &[u8]) -> Result<CoreCall, KernelError> {
    if payload.is_empty() {
        return Err(KernelError::InvalidTransaction("empty payload"));
    }

    match payload[0] {
        0x00 => Ok(CoreCall::Noop),
        0x01 => {
            if payload.len() < 3 {
                return Err(KernelError::InvalidTransaction(
                    "set payload too short for lengths",
                ));
            }
            let key_len = payload[1] as usize;
            let value_len = payload[2] as usize;
            let expected = 3usize
                .checked_add(key_len)
                .and_then(|v| v.checked_add(value_len))
                .ok_or(KernelError::InvalidTransaction("payload length overflow"))?;
            if expected != payload.len() {
                return Err(KernelError::InvalidTransaction("set payload malformed"));
            }
            let key = payload[3..3 + key_len].to_vec();
            let value = payload[3 + key_len..expected].to_vec();
            Ok(CoreCall::Set { key, value })
        }
        0x02 => {
            if payload.len() < 2 {
                return Err(KernelError::InvalidTransaction(
                    "delete payload too short for key length",
                ));
            }
            let key_len = payload[1] as usize;
            let expected = 2usize
                .checked_add(key_len)
                .ok_or(KernelError::InvalidTransaction("payload length overflow"))?;
            if expected != payload.len() {
                return Err(KernelError::InvalidTransaction("delete payload malformed"));
            }
            Ok(CoreCall::Delete {
                key: payload[2..expected].to_vec(),
            })
        }
        _ => Err(KernelError::DeterministicAbort {
            code: 1,
            message: "unsupported operation",
        }),
    }
}

/// Native deterministic kernel lane with a minimal call surface.
pub struct CoreLaneAdapter;

impl<S: HostState> LaneAdapter<S> for CoreLaneAdapter {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError> {
        match decode_core_call(&env.tx.payload)? {
            CoreCall::Noop => {
                let event = env.emit_event(b"core.noop".to_vec(), Vec::new())?;
                Ok(LaneOutput {
                    output: b"noop".to_vec(),
                    events: vec![event],
                })
            }
            CoreCall::Set { key, value } => {
                env.write_state(key.clone(), value.clone())?;
                let event = env.emit_event(b"core.set".to_vec(), key)?;
                Ok(LaneOutput {
                    output: value,
                    events: vec![event],
                })
            }
            CoreCall::Delete { key } => {
                env.delete_state(key.clone())?;
                let event = env.emit_event(b"core.delete".to_vec(), key)?;
                Ok(LaneOutput {
                    output: b"deleted".to_vec(),
                    events: vec![event],
                })
            }
        }
    }
}

/// Compatibility lane wrapper for EVM family runtimes.
pub struct EvmCompatibilityLane;

impl<S: HostState> LaneAdapter<S> for EvmCompatibilityLane {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError> {
        let event = env.emit_event(b"lane.evm.dispatched".to_vec(), env.tx.tx_id.to_vec())?;
        Ok(LaneOutput {
            output: env.tx.payload.clone(),
            events: vec![event],
        })
    }
}

/// Compatibility lane wrapper for WASM family runtimes.
pub struct WasmCompatibilityLane;

impl<S: HostState> LaneAdapter<S> for WasmCompatibilityLane {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError> {
        let event = env.emit_event(b"lane.wasm.dispatched".to_vec(), env.tx.tx_id.to_vec())?;
        Ok(LaneOutput {
            output: env.tx.payload.clone(),
            events: vec![event],
        })
    }
}

pub struct LaneRegistry<S: HostState> {
    adapters: BTreeMap<LaneId, Box<dyn LaneAdapter<S> + Send + Sync>>,
}

impl<S: HostState> Default for LaneRegistry<S> {
    fn default() -> Self {
        Self {
            adapters: BTreeMap::new(),
        }
    }
}

impl<S: HostState> LaneRegistry<S> {
    pub fn register<A>(&mut self, lane: LaneId, adapter: A)
    where
        A: LaneAdapter<S> + Send + Sync + 'static,
    {
        self.adapters.insert(lane, Box::new(adapter));
    }

    pub fn resolve(&self, lane: LaneId) -> Option<&(dyn LaneAdapter<S> + Send + Sync)> {
        self.adapters.get(&lane).map(Box::as_ref)
    }

    pub fn lanes(&self) -> BTreeSet<LaneId> {
        self.adapters.keys().copied().collect()
    }

    pub fn with_default_compatibility_lanes() -> Self
    where
        S: 'static,
    {
        let mut registry = Self::default();
        registry.register(LaneId::Core, CoreLaneAdapter);
        registry.register(LaneId::Evm, EvmCompatibilityLane);
        registry.register(LaneId::Wasm, WasmCompatibilityLane);
        registry
    }
}

pub struct CoreKernel<S: HostState> {
    schedule: FuelSchedule,
    lanes: LaneRegistry<S>,
}

impl<S: HostState> CoreKernel<S> {
    pub fn new(schedule: FuelSchedule, lanes: LaneRegistry<S>) -> Self {
        Self { schedule, lanes }
    }

    pub fn execute_tx(
        &self,
        block: &BlockExecutionContext,
        tx: &CanonicalTxEnvelope,
        state: &mut S,
    ) -> Receipt {
        if let Err(err) = tx.validate() {
            return Receipt {
                tx_id: tx.tx_id,
                lane: tx.lane,
                success: false,
                gas_used: 0,
                events: Vec::new(),
                output: Vec::new(),
                error: Some(err),
            };
        }

        let mut fuel = FuelMeter::new(tx.gas_limit);
        if let Err(err) = self.charge_intrinsic_cost(&mut fuel, tx) {
            return Receipt {
                tx_id: tx.tx_id,
                lane: tx.lane,
                success: false,
                gas_used: fuel.used(),
                events: Vec::new(),
                output: Vec::new(),
                error: Some(err),
            };
        }

        let Some(adapter) = self.lanes.resolve(tx.lane) else {
            return Receipt {
                tx_id: tx.tx_id,
                lane: tx.lane,
                success: false,
                gas_used: fuel.used(),
                events: Vec::new(),
                output: Vec::new(),
                error: Some(KernelError::LaneNotRegistered(tx.lane)),
            };
        };

        let mut journal = StateJournal::new();
        let mut env = ExecutionEnv {
            block,
            tx,
            state,
            journal: &mut journal,
            fuel: &mut fuel,
            schedule: &self.schedule,
        };

        match adapter.execute(&mut env) {
            Ok(lane_out) => {
                journal.commit(state);
                Receipt {
                    tx_id: tx.tx_id,
                    lane: tx.lane,
                    success: true,
                    gas_used: fuel.used(),
                    events: lane_out.events,
                    output: lane_out.output,
                    error: None,
                }
            }
            Err(err) => {
                journal.revert();
                Receipt {
                    tx_id: tx.tx_id,
                    lane: tx.lane,
                    success: false,
                    gas_used: fuel.used(),
                    events: Vec::new(),
                    output: Vec::new(),
                    error: Some(err),
                }
            }
        }
    }

    fn charge_intrinsic_cost(
        &self,
        fuel: &mut FuelMeter,
        tx: &CanonicalTxEnvelope,
    ) -> Result<(), KernelError> {
        let payload_len = u64::try_from(tx.payload.len())
            .map_err(|_| KernelError::InvalidTransaction("payload too large"))?;
        fuel.charge(self.schedule.tx_base)?;
        fuel.charge(payload_len.saturating_mul(self.schedule.byte_cost))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryState(BTreeMap<StateKey, StateValue>);

    impl HostState for MemoryState {
        fn get(&self, key: &[u8]) -> Option<StateValue> {
            self.0.get(key).cloned()
        }

        fn set(&mut self, key: StateKey, value: StateValue) {
            self.0.insert(key, value);
        }

        fn delete(&mut self, key: &[u8]) {
            self.0.remove(key);
        }
    }

    struct CoreLane;

    impl LaneAdapter<MemoryState> for CoreLane {
        fn execute(
            &self,
            env: &mut ExecutionEnv<'_, MemoryState>,
        ) -> Result<LaneOutput, KernelError> {
            env.write_state(b"counter".to_vec(), b"1".to_vec())?;
            let event = env.emit_event(b"core.executed".to_vec(), env.tx.payload.clone())?;
            Ok(LaneOutput {
                output: b"ok".to_vec(),
                events: vec![event],
            })
        }
    }

    struct FailingLane;

    impl LaneAdapter<MemoryState> for FailingLane {
        fn execute(
            &self,
            env: &mut ExecutionEnv<'_, MemoryState>,
        ) -> Result<LaneOutput, KernelError> {
            env.write_state(b"counter".to_vec(), b"2".to_vec())?;
            Err(KernelError::DeterministicAbort {
                code: 7,
                message: "lane failed",
            })
        }
    }

    fn sample_block() -> BlockExecutionContext {
        BlockExecutionContext {
            chain_id: 4242,
            block_number: 1,
            block_timestamp: 1_700_000_000,
            proposer: [1u8; 32],
            base_fee: 1,
        }
    }

    fn sample_tx(lane: LaneId, gas_limit: Gas) -> CanonicalTxEnvelope {
        CanonicalTxEnvelope {
            tx_id: [9u8; 32],
            sender: b"alice".to_vec(),
            nonce: 1,
            lane,
            gas_limit,
            max_fee_per_gas: 1,
            payload: b"hello".to_vec(),
            signature: vec![1, 2, 3],
        }
    }

    fn encode_set_payload(key: &[u8], value: &[u8]) -> Vec<u8> {
        let key_len = u8::try_from(key.len()).expect("test key length should fit into u8");
        let value_len = u8::try_from(value.len()).expect("test value length should fit into u8");
        let mut payload = vec![0x01, key_len, value_len];
        payload.extend_from_slice(key);
        payload.extend_from_slice(value);
        payload
    }

    #[test]
    fn commits_journal_on_success() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, CoreLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        let receipt = kernel.execute_tx(
            &sample_block(),
            &sample_tx(LaneId::Core, 200_000),
            &mut state,
        );

        assert!(receipt.success);
        assert_eq!(state.get(b"counter"), Some(b"1".to_vec()));
        assert_eq!(receipt.events.len(), 1);
    }

    #[test]
    fn reverts_journal_on_failure() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, FailingLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        state.set(b"counter".to_vec(), b"0".to_vec());

        let receipt = kernel.execute_tx(
            &sample_block(),
            &sample_tx(LaneId::Core, 200_000),
            &mut state,
        );

        assert!(!receipt.success);
        assert_eq!(state.get(b"counter"), Some(b"0".to_vec()));
    }

    #[test]
    fn fails_deterministically_when_out_of_fuel() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, CoreLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        let receipt = kernel.execute_tx(&sample_block(), &sample_tx(LaneId::Core, 1), &mut state);

        assert!(!receipt.success);
        assert_eq!(receipt.error, Some(KernelError::GasExhausted));
    }

    #[test]
    fn default_registry_includes_core_evm_wasm() {
        let lanes = LaneRegistry::<MemoryState>::with_default_compatibility_lanes();
        let names = lanes.lanes();
        assert!(names.contains(&LaneId::Core));
        assert!(names.contains(&LaneId::Evm));
        assert!(names.contains(&LaneId::Wasm));
    }

    #[test]
    fn core_compatibility_lane_can_set_state() {
        let lanes = LaneRegistry::<MemoryState>::with_default_compatibility_lanes();
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);
        let mut state = MemoryState::default();

        let mut tx = sample_tx(LaneId::Core, 300_000);
        tx.payload = encode_set_payload(b"k", b"v");
        let receipt = kernel.execute_tx(&sample_block(), &tx, &mut state);

        assert!(receipt.success);
        assert_eq!(state.get(b"k"), Some(b"v".to_vec()));
        assert_eq!(receipt.events.len(), 1);
        assert_eq!(receipt.events[0].topic, b"core.set".to_vec());
    }

    #[test]
    fn evm_and_wasm_are_compatibility_lanes() {
        let lanes = LaneRegistry::<MemoryState>::with_default_compatibility_lanes();
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);
        let mut state = MemoryState::default();

        let evm_tx = sample_tx(LaneId::Evm, 300_000);
        let wasm_tx = sample_tx(LaneId::Wasm, 300_000);

        let evm_receipt = kernel.execute_tx(&sample_block(), &evm_tx, &mut state);
        let wasm_receipt = kernel.execute_tx(&sample_block(), &wasm_tx, &mut state);

        assert!(evm_receipt.success);
        assert!(wasm_receipt.success);
        assert_eq!(evm_receipt.events[0].topic, b"lane.evm.dispatched".to_vec());
        assert_eq!(
            wasm_receipt.events[0].topic,
            b"lane.wasm.dispatched".to_vec()
        );
    }
}
