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
}
