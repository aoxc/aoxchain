// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeMap, BTreeSet};

use crate::gas::Gas;

pub type StateKey = Vec<u8>;
pub type StateValue = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTxEnvelope {
    pub tx_id: [u8; 32],
    pub sender: Vec<u8>,
    pub nonce: u64,
    pub fee: CanonicalFee,
    pub lane: LaneId,
    pub gas_limit: Gas,
    pub max_fee_per_gas: u128,
    pub payload: Vec<u8>,
    pub auth_proof: AuthProof,
    pub intent_flags: IntentFlags,
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
        if self.auth_proof.is_empty() {
            return Err(KernelError::InvalidTransaction(
                "auth_proof must not be empty",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalFee {
    pub amount: u128,
    pub asset: Vec<u8>,
}

impl Default for CanonicalFee {
    fn default() -> Self {
        Self {
            amount: 0,
            asset: b"AOXC".to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthProof {
    pub scheme: Vec<u8>,
    pub proof: Vec<u8>,
}

impl AuthProof {
    pub fn is_empty(&self) -> bool {
        self.scheme.is_empty() || self.proof.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IntentFlags {
    bits: u16,
}

impl IntentFlags {
    pub const CROSS_LANE: u16 = 0b0001;
    pub const QUERY_ONLY: u16 = 0b0010;
    pub const SETTLEMENT_ONLY: u16 = 0b0100;

    pub fn new(bits: u16) -> Self {
        Self { bits }
    }

    pub fn bits(&self) -> u16 {
        self.bits
    }

    pub fn contains(&self, flag: u16) -> bool {
        (self.bits & flag) == flag
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
    Move,
    Other(u16),
}

impl LaneId {
    fn commitment_discriminant(self) -> [u8; 2] {
        match self {
            LaneId::Core => [0, 1],
            LaneId::Evm => [0, 2],
            LaneId::Wasm => [0, 3],
            LaneId::Move => [0, 4],
            LaneId::Other(value) => value.to_le_bytes(),
        }
    }
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
    pub state_diff_hash: [u8; 32],
    pub receipt_hash: [u8; 32],
    pub error: Option<KernelError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedReceipt {
    pub success: bool,
    pub gas_used: Gas,
    pub events: Vec<Event>,
    pub output: Vec<u8>,
    pub state_diff_hash: [u8; 32],
    pub receipt_hash: [u8; 32],
}

impl Receipt {
    pub fn unified(&self) -> UnifiedReceipt {
        UnifiedReceipt {
            success: self.success,
            gas_used: self.gas_used,
            events: self.events.clone(),
            output: self.output.clone(),
            state_diff_hash: self.state_diff_hash,
            receipt_hash: self.receipt_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateCommitment {
    pub storage_root: [u8; 32],
    pub execution_diff_root: [u8; 32],
    pub receipt_root: [u8; 32],
    pub lane_commitment_root: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityProof {
    pub proof_type: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateTransitionResult {
    pub receipt: Receipt,
    pub commitment: StateCommitment,
    pub finality_proof: FinalityProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    InvalidTransaction(&'static str),
    LaneNotRegistered(LaneId),
    GasExhausted,
    StateViolation(&'static str),
    DeterministicAbort { code: u16, message: &'static str },
    AdapterValidation(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterQuery {
    pub lane: LaneId,
    pub method: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterQueryResult {
    pub output: Vec<u8>,
    pub deterministic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterExecutionOutput {
    pub output: Vec<u8>,
    pub events: Vec<Event>,
    pub state_diff_hash: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneCapabilityManifest {
    pub account_model: Vec<u8>,
    pub state_model: Vec<u8>,
    pub gas_model: Vec<u8>,
    pub event_model: Vec<u8>,
    pub contract_lifecycle: Vec<u8>,
    pub cross_lane_support: bool,
    pub determinism_level: u8,
    pub compatibility_tier: CompatibilityTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityTier {
    Tier1NativeFull,
    Tier2High,
    Tier3Adapter,
    Tier4SettlementOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicSandboxPolicy {
    pub wall_clock_allowed: bool,
    pub random_host_allowed: bool,
    pub max_memory_bytes: u64,
    pub max_gas: Gas,
    pub max_syscalls: u32,
    pub deterministic_io_only: bool,
}

impl Default for DeterministicSandboxPolicy {
    fn default() -> Self {
        Self {
            wall_clock_allowed: false,
            random_host_allowed: false,
            max_memory_bytes: 128 * 1024 * 1024,
            max_gas: 20_000_000,
            max_syscalls: 10_000,
            deterministic_io_only: true,
        }
    }
}

impl DeterministicSandboxPolicy {
    pub fn validate_tx(&self, tx: &CanonicalTxEnvelope) -> Result<(), KernelError> {
        if self.wall_clock_allowed || self.random_host_allowed || !self.deterministic_io_only {
            return Err(KernelError::StateViolation(
                "sandbox policy must remain deterministic",
            ));
        }
        if tx.gas_limit > self.max_gas {
            return Err(KernelError::InvalidTransaction(
                "gas_limit exceeds deterministic sandbox max",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossLaneMessage {
    pub version: u16,
    pub source_lane: LaneId,
    pub target_lane: LaneId,
    pub tx_id: [u8; 32],
    pub sequence: u64,
    pub payload: Vec<u8>,
}

impl CrossLaneMessage {
    pub fn replay_key(&self) -> (LaneId, LaneId, [u8; 32], u64, u16) {
        (
            self.source_lane,
            self.target_lane,
            self.tx_id,
            self.sequence,
            self.version,
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrossLaneBus {
    queue: Vec<CrossLaneMessage>,
    seen: BTreeSet<(LaneId, LaneId, [u8; 32], u64, u16)>,
}

impl CrossLaneBus {
    pub fn enqueue(&mut self, message: CrossLaneMessage) -> Result<(), KernelError> {
        if !self.seen.insert(message.replay_key()) {
            return Err(KernelError::DeterministicAbort {
                code: 77,
                message: "cross-lane replay detected",
            });
        }
        self.queue.push(message);
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<CrossLaneMessage> {
        std::mem::take(&mut self.queue)
    }
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

pub trait ExecutionAdapter<S: HostState> {
    fn manifest(&self) -> &LaneCapabilityManifest;
    fn validate(&self, env: &ExecutionEnv<'_, S>) -> Result<(), KernelError>;
    fn execute(&self, env: &mut ExecutionEnv<'_, S>)
    -> Result<AdapterExecutionOutput, KernelError>;
    fn query(&self, query: &AdapterQuery) -> Result<AdapterQueryResult, KernelError>;
    fn export_receipt(&self, output: &AdapterExecutionOutput, env: &ExecutionEnv<'_, S>)
    -> Receipt;
    fn export_state_commitment(
        &self,
        output: &AdapterExecutionOutput,
        receipt: &Receipt,
        env: &ExecutionEnv<'_, S>,
    ) -> StateCommitment;
}

pub trait LaneAdapter<S: HostState> {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError>;
}

impl<S: HostState, T: ExecutionAdapter<S>> LaneAdapter<S> for T {
    fn execute(&self, env: &mut ExecutionEnv<'_, S>) -> Result<LaneOutput, KernelError> {
        self.validate(env)?;
        let out = <T as ExecutionAdapter<S>>::execute(self, env)?;
        Ok(LaneOutput {
            output: out.output,
            events: out.events,
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
}

pub struct CoreKernel<S: HostState> {
    schedule: FuelSchedule,
    lanes: LaneRegistry<S>,
    sandbox_policy: DeterministicSandboxPolicy,
}

impl<S: HostState> CoreKernel<S> {
    pub fn new(schedule: FuelSchedule, lanes: LaneRegistry<S>) -> Self {
        Self::with_policy(schedule, lanes, DeterministicSandboxPolicy::default())
    }

    pub fn with_policy(
        schedule: FuelSchedule,
        lanes: LaneRegistry<S>,
        sandbox_policy: DeterministicSandboxPolicy,
    ) -> Self {
        Self {
            schedule,
            lanes,
            sandbox_policy,
        }
    }

    pub fn execute_tx(
        &self,
        block: &BlockExecutionContext,
        tx: &CanonicalTxEnvelope,
        state: &mut S,
    ) -> Receipt {
        if let Err(err) = tx.validate() {
            return self.failure_receipt(tx, 0, err);
        }
        if let Err(err) = self.sandbox_policy.validate_tx(tx) {
            return self.failure_receipt(tx, 0, err);
        }

        let mut fuel = FuelMeter::new(tx.gas_limit);
        if let Err(err) = self.charge_intrinsic_cost(&mut fuel, tx) {
            return self.failure_receipt(tx, fuel.used(), err);
        }

        let Some(adapter) = self.lanes.resolve(tx.lane) else {
            return self.failure_receipt(tx, fuel.used(), KernelError::LaneNotRegistered(tx.lane));
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
                let state_diff_hash = derive_hash32(b"AOXC-STATE-DIFF", &lane_out.output);
                let receipt_hash = derive_hash32(b"AOXC-RECEIPT", &lane_out.output);
                journal.commit(state);
                Receipt {
                    tx_id: tx.tx_id,
                    lane: tx.lane,
                    success: true,
                    gas_used: fuel.used(),
                    events: lane_out.events,
                    output: lane_out.output,
                    state_diff_hash,
                    receipt_hash,
                    error: None,
                }
            }
            Err(err) => {
                journal.revert();
                self.failure_receipt(tx, fuel.used(), err)
            }
        }
    }

    pub fn execute_tx_with_commitment(
        &self,
        block: &BlockExecutionContext,
        tx: &CanonicalTxEnvelope,
        state: &mut S,
    ) -> StateTransitionResult {
        let receipt = self.execute_tx(block, tx, state);
        let lane_tag = tx.lane.commitment_discriminant();
        let commitment = StateCommitment {
            storage_root: derive_hash32(b"AOXC-STORAGE-ROOT", &receipt.output),
            execution_diff_root: receipt.state_diff_hash,
            receipt_root: receipt.receipt_hash,
            lane_commitment_root: derive_hash32(b"AOXC-LANE-ROOT", &lane_tag),
        };
        let finality_proof = FinalityProof {
            proof_type: b"deterministic-mock-finality-v1".to_vec(),
            payload: derive_hash32(b"AOXC-FINALITY", &receipt.receipt_hash).to_vec(),
        };

        StateTransitionResult {
            receipt,
            commitment,
            finality_proof,
        }
    }

    pub fn conformance_replay(
        &self,
        block: &BlockExecutionContext,
        tx: &CanonicalTxEnvelope,
        state_factory: impl Fn() -> S,
    ) -> Result<(), KernelError> {
        let mut state_a = state_factory();
        let mut state_b = state_factory();
        let first = self.execute_tx_with_commitment(block, tx, &mut state_a);
        let second = self.execute_tx_with_commitment(block, tx, &mut state_b);

        if first.receipt != second.receipt {
            return Err(KernelError::StateViolation(
                "conformance failed: same input produced different receipt",
            ));
        }
        if first.commitment != second.commitment {
            return Err(KernelError::StateViolation(
                "conformance failed: same receipt produced different commitment",
            ));
        }
        Ok(())
    }

    fn failure_receipt(
        &self,
        tx: &CanonicalTxEnvelope,
        gas_used: Gas,
        error: KernelError,
    ) -> Receipt {
        Receipt {
            tx_id: tx.tx_id,
            lane: tx.lane,
            success: false,
            gas_used,
            events: Vec::new(),
            output: Vec::new(),
            state_diff_hash: [0u8; 32],
            receipt_hash: derive_hash32(b"AOXC-RECEIPT-ERR", format!("{:?}", error).as_bytes()),
            error: Some(error),
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

fn derive_hash32(domain: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (index, byte) in domain.iter().chain(payload.iter()).enumerate() {
        out[index % 32] ^= *byte;
        out[(index * 7) % 32] = out[(index * 7) % 32].wrapping_add(*byte);
    }
    out
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
            fee: CanonicalFee::default(),
            lane,
            gas_limit,
            max_fee_per_gas: 1,
            payload: b"hello".to_vec(),
            auth_proof: AuthProof {
                scheme: b"ed25519".to_vec(),
                proof: vec![1, 2, 3, 4],
            },
            intent_flags: IntentFlags::new(IntentFlags::CROSS_LANE),
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
        assert_ne!(receipt.receipt_hash, [0u8; 32]);
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
    fn detects_cross_lane_replay() {
        let mut bus = CrossLaneBus::default();
        let message = CrossLaneMessage {
            version: 1,
            source_lane: LaneId::Evm,
            target_lane: LaneId::Wasm,
            tx_id: [5u8; 32],
            sequence: 42,
            payload: b"bridge".to_vec(),
        };

        assert!(bus.enqueue(message.clone()).is_ok());
        assert!(matches!(
            bus.enqueue(message),
            Err(KernelError::DeterministicAbort { code: 77, .. })
        ));
    }

    #[test]
    fn conformance_replay_same_input_same_receipt_and_commitment() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, CoreLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        assert!(
            kernel
                .conformance_replay(&sample_block(), &sample_tx(LaneId::Core, 200_000), || {
                    MemoryState::default()
                })
                .is_ok()
        );
    }
}
