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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecurityFlag {
    CapabilityGatedHost,
    DeterministicReplayAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSettlementReceipt {
    pub tx_id: [u8; 32],
    pub lane: LaneId,
    pub status: CanonicalStatus,
    pub gas_used: Gas,
    pub state_diff_hash: [u8; 32],
    pub receipt_hash: [u8; 32],
    pub event_count: u32,
    pub replay_hash: [u8; 32],
    pub execution_trace_hash: [u8; 32],
    pub security_flags: BTreeSet<SecurityFlag>,
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

    pub fn canonical(&self) -> CanonicalSettlementReceipt {
        let status = if self.success {
            CanonicalStatus::Success
        } else {
            CanonicalStatus::Failure
        };

        let event_count = u32::try_from(self.events.len()).unwrap_or(u32::MAX);
        let replay_hash = derive_hash32(b"AOXC-REPLAY", &self.receipt_hash);
        let execution_trace_hash = derive_execution_trace_hash(&self.events, &self.output);
        let security_flags = BTreeSet::from([
            SecurityFlag::CapabilityGatedHost,
            SecurityFlag::DeterministicReplayAnchor,
        ]);

        CanonicalSettlementReceipt {
            tx_id: self.tx_id,
            lane: self.lane,
            status,
            gas_used: self.gas_used,
            state_diff_hash: self.state_diff_hash,
            receipt_hash: self.receipt_hash,
            event_count,
            replay_hash,
            execution_trace_hash,
            security_flags,
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
    DeterministicAbort {
        code: u16,
        message: &'static str,
    },
    AdapterValidation(&'static str),
    CapabilityDenied {
        lane: LaneId,
        capability: HostCapability,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostCapability {
    StorageRead,
    StorageWrite,
    EventEmit,
    CrossLaneCall,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapabilityProfile {
    allowed: BTreeSet<HostCapability>,
}

impl CapabilityProfile {
    pub fn strict(allowed: impl IntoIterator<Item = HostCapability>) -> Self {
        Self {
            allowed: allowed.into_iter().collect(),
        }
    }

    pub fn allows(&self, capability: HostCapability) -> bool {
        self.allowed.contains(&capability)
    }

    pub fn all() -> Self {
        Self::strict([
            HostCapability::StorageRead,
            HostCapability::StorageWrite,
            HostCapability::EventEmit,
            HostCapability::CrossLaneCall,
        ])
    }
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
    expected_sequence: BTreeMap<(LaneId, LaneId, [u8; 32], u16), u64>,
}

impl CrossLaneBus {
    pub fn enqueue(&mut self, message: CrossLaneMessage) -> Result<(), KernelError> {
        if !self.seen.insert(message.replay_key()) {
            return Err(KernelError::DeterministicAbort {
                code: 77,
                message: "cross-lane replay detected",
            });
        }
        let causal_key = (
            message.source_lane,
            message.target_lane,
            message.tx_id,
            message.version,
        );
        let expected = *self.expected_sequence.get(&causal_key).unwrap_or(&0);
        if message.sequence != expected {
            self.seen.remove(&message.replay_key());
            return Err(KernelError::DeterministicAbort {
                code: 78,
                message: "cross-lane causal ordering violation",
            });
        }
        self.expected_sequence
            .insert(causal_key, expected.saturating_add(1));
        self.queue.push(message);
        Ok(())
    }

    pub fn drain(&mut self) -> Vec<CrossLaneMessage> {
        std::mem::take(&mut self.queue)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneExecutionIntent {
    pub lane: LaneId,
    pub tx_id: [u8; 32],
    pub declared_reads: BTreeSet<StateKey>,
    pub declared_writes: BTreeSet<StateKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicExecutionPlan {
    pub serial_order: Vec<[u8; 32]>,
    pub parallel_batches: Vec<Vec<[u8; 32]>>,
}

pub fn plan_deterministic_batches(
    mut intents: Vec<LaneExecutionIntent>,
) -> DeterministicExecutionPlan {
    intents.sort_by_key(|intent| (intent.lane, intent.tx_id));

    let serial_order = intents
        .iter()
        .map(|intent| intent.tx_id)
        .collect::<Vec<_>>();
    let mut parallel_batches: Vec<Vec<[u8; 32]>> = Vec::new();
    let mut batch_reads: Vec<BTreeSet<StateKey>> = Vec::new();
    let mut batch_writes: Vec<BTreeSet<StateKey>> = Vec::new();

    for intent in intents {
        let mut placed = false;
        for index in 0..parallel_batches.len() {
            let has_write_conflict = !intent.declared_writes.is_disjoint(&batch_writes[index]);
            let has_read_write_overlap = !intent.declared_reads.is_disjoint(&batch_writes[index])
                || !intent.declared_writes.is_disjoint(&batch_reads[index]);

            if !has_write_conflict && !has_read_write_overlap {
                parallel_batches[index].push(intent.tx_id);
                batch_reads[index].extend(intent.declared_reads.clone());
                batch_writes[index].extend(intent.declared_writes.clone());
                placed = true;
                break;
            }
        }

        if !placed {
            parallel_batches.push(vec![intent.tx_id]);
            batch_reads.push(intent.declared_reads);
            batch_writes.push(intent.declared_writes);
        }
    }

    DeterministicExecutionPlan {
        serial_order,
        parallel_batches,
    }
}

pub trait HostState {
    fn get(&self, key: &[u8]) -> Option<StateValue>;
    fn set(&mut self, key: StateKey, value: StateValue);
    fn delete(&mut self, key: &[u8]);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalOp {
    Put {
        lane: LaneId,
        key: StateKey,
        value: StateValue,
    },
    Delete {
        lane: LaneId,
        key: StateKey,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StateJournal {
    ops: Vec<JournalOp>,
    checkpoints: Vec<usize>,
}

impl StateJournal {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin_transaction(&mut self) {
        self.ops.clear();
        self.checkpoints.clear();
    }

    pub fn checkpoint(&mut self) -> usize {
        let marker = self.ops.len();
        self.checkpoints.push(marker);
        marker
    }

    pub fn rollback(&mut self, checkpoint: usize) -> Result<(), KernelError> {
        if checkpoint > self.ops.len() {
            return Err(KernelError::StateViolation("invalid journal checkpoint"));
        }
        self.ops.truncate(checkpoint);
        self.checkpoints.retain(|marker| *marker <= checkpoint);
        Ok(())
    }

    pub fn put(&mut self, lane: LaneId, key: StateKey, value: StateValue) {
        self.ops.push(JournalOp::Put { lane, key, value });
    }

    pub fn delete(&mut self, lane: LaneId, key: StateKey) {
        self.ops.push(JournalOp::Delete { lane, key });
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
                JournalOp::Put { key, value, .. } => state.set(key, value),
                JournalOp::Delete { key, .. } => state.delete(&key),
            }
        }
    }

    pub fn revert(self) {}

    pub fn lane_conflicts(&self) -> BTreeSet<StateKey> {
        let mut owners: BTreeMap<&[u8], LaneId> = BTreeMap::new();
        let mut conflicts = BTreeSet::new();
        for op in &self.ops {
            let (lane, key) = match op {
                JournalOp::Put { lane, key, .. } => (*lane, key.as_slice()),
                JournalOp::Delete { lane, key } => (*lane, key.as_slice()),
            };
            if let Some(existing) = owners.get(key) {
                if *existing != lane {
                    conflicts.insert(key.to_vec());
                }
            } else {
                owners.insert(key, lane);
            }
        }
        conflicts
    }
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
    pub capabilities: &'a CapabilityProfile,
}

impl<'a, S: HostState> ExecutionEnv<'a, S> {
    fn scoped_state_key(&self, key: &[u8]) -> StateKey {
        if key.starts_with(b"shared/") {
            return key.to_vec();
        }

        let mut scoped = b"lane/".to_vec();
        scoped.extend_from_slice(&self.tx.lane.commitment_discriminant());
        scoped.push(b'/');
        scoped.extend_from_slice(key);
        scoped
    }

    pub fn read_state(&mut self, key: &[u8]) -> Option<StateValue> {
        for op in self.journal.ops.iter().rev() {
            match op {
                JournalOp::Put {
                    key: journal_key,
                    value,
                    ..
                } if journal_key.as_slice() == key => {
                    return Some(value.clone());
                }
                JournalOp::Delete {
                    key: journal_key, ..
                } if journal_key.as_slice() == key => {
                    return None;
                }
                _ => {}
            }
        }
        self.state.get(key)
    }

    pub fn write_state(&mut self, key: StateKey, value: StateValue) -> Result<(), KernelError> {
        if !self.capabilities.allows(HostCapability::StorageWrite) {
            return Err(KernelError::CapabilityDenied {
                lane: self.tx.lane,
                capability: HostCapability::StorageWrite,
            });
        }
        self.fuel.charge(self.schedule.state_write_cost)?;
        self.journal.put(self.tx.lane, key, value);
        Ok(())
    }

    pub fn delete_state(&mut self, key: StateKey) -> Result<(), KernelError> {
        if !self.capabilities.allows(HostCapability::StorageWrite) {
            return Err(KernelError::CapabilityDenied {
                lane: self.tx.lane,
                capability: HostCapability::StorageWrite,
            });
        }
        self.fuel.charge(self.schedule.state_delete_cost)?;
        self.journal.delete(self.tx.lane, key);
        Ok(())
    }

    pub fn emit_event(&mut self, topic: Vec<u8>, data: Vec<u8>) -> Result<Event, KernelError> {
        if !self.capabilities.allows(HostCapability::EventEmit) {
            return Err(KernelError::CapabilityDenied {
                lane: self.tx.lane,
                capability: HostCapability::EventEmit,
            });
        }
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
    adapters: BTreeMap<LaneId, LaneRegistration<S>>,
}

pub struct LaneRegistration<S: HostState> {
    adapter: Box<dyn LaneAdapter<S> + Send + Sync>,
    capabilities: CapabilityProfile,
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
        self.register_with_capabilities(lane, adapter, CapabilityProfile::all());
    }

    pub fn register_with_capabilities<A>(
        &mut self,
        lane: LaneId,
        adapter: A,
        capabilities: CapabilityProfile,
    ) where
        A: LaneAdapter<S> + Send + Sync + 'static,
    {
        self.adapters.insert(
            lane,
            LaneRegistration {
                adapter: Box::new(adapter),
                capabilities,
            },
        );
    }

    pub fn resolve(&self, lane: LaneId) -> Option<&LaneRegistration<S>> {
        self.adapters.get(&lane)
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

        let Some(registration) = self.lanes.resolve(tx.lane) else {
            return self.failure_receipt(tx, fuel.used(), KernelError::LaneNotRegistered(tx.lane));
        };

        let mut journal = StateJournal::new();
        journal.begin_transaction();
        let mut env = ExecutionEnv {
            block,
            tx,
            state,
            journal: &mut journal,
            fuel: &mut fuel,
            schedule: &self.schedule,
            capabilities: &registration.capabilities,
        };

        match registration.adapter.execute(&mut env) {
            Ok(lane_out) => {
                if !journal.lane_conflicts().is_empty() {
                    return self.failure_receipt(
                        tx,
                        fuel.used(),
                        KernelError::StateViolation("cross-lane write conflict detected"),
                    );
                }
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

    pub fn execute_tx_canonical(
        &self,
        block: &BlockExecutionContext,
        tx: &CanonicalTxEnvelope,
        state: &mut S,
    ) -> CanonicalSettlementReceipt {
        self.execute_tx(block, tx, state).canonical()
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

fn derive_execution_trace_hash(events: &[Event], output: &[u8]) -> [u8; 32] {
    let mut material = Vec::new();
    for event in events {
        material.extend_from_slice(&event.topic);
        material.extend_from_slice(&event.data);
    }
    material.extend_from_slice(output);
    derive_hash32(b"AOXC-TRACE", &material)
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

    fn lane_scoped_key(lane: LaneId, key: &[u8]) -> Vec<u8> {
        let mut scoped = b"lane/".to_vec();
        scoped.extend_from_slice(&lane.commitment_discriminant());
        scoped.push(b'/');
        scoped.extend_from_slice(key);
        scoped
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
        assert_eq!(
            state.get(&lane_scoped_key(LaneId::Core, b"counter")),
            Some(b"1".to_vec())
        );
        assert_eq!(receipt.events.len(), 1);
        assert_ne!(receipt.receipt_hash, [0u8; 32]);
    }

    #[test]
    fn reverts_journal_on_failure() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, FailingLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        state.set(lane_scoped_key(LaneId::Core, b"counter"), b"0".to_vec());

        let receipt = kernel.execute_tx(
            &sample_block(),
            &sample_tx(LaneId::Core, 200_000),
            &mut state,
        );

        assert!(!receipt.success);
        assert_eq!(
            state.get(&lane_scoped_key(LaneId::Core, b"counter")),
            Some(b"0".to_vec())
        );
    }

    #[test]
    fn journal_checkpoint_and_rollback_work() {
        let mut journal = StateJournal::new();
        journal.begin_transaction();
        journal.put(LaneId::Core, b"a".to_vec(), b"1".to_vec());
        let checkpoint = journal.checkpoint();
        journal.put(LaneId::Core, b"a".to_vec(), b"2".to_vec());
        journal
            .rollback(checkpoint)
            .expect("rollback to checkpoint should succeed");

        let mut state = MemoryState::default();
        journal.commit(&mut state);
        assert_eq!(state.get(b"a"), Some(b"1".to_vec()));
    }

    #[test]
    fn read_state_prefers_journal_overlay() {
        let mut state = MemoryState::default();
        state.set(b"counter".to_vec(), b"1".to_vec());

        let block = sample_block();
        let tx = sample_tx(LaneId::Core, 200_000);
        let mut journal = StateJournal::new();
        journal.begin_transaction();
        let mut fuel = FuelMeter::new(200_000);
        let schedule = FuelSchedule::default();
        let mut env = ExecutionEnv {
            block: &block,
            tx: &tx,
            state: &mut state,
            journal: &mut journal,
            fuel: &mut fuel,
            schedule: &schedule,
            capabilities: &CapabilityProfile::all(),
        };

        env.write_state(b"counter".to_vec(), b"9".to_vec())
            .expect("write should succeed");
        assert_eq!(env.read_state(b"counter"), Some(b"9".to_vec()));
    }

    #[test]
    fn lane_without_storage_write_capability_is_blocked() {
        let mut lanes = LaneRegistry::default();
        lanes.register_with_capabilities(
            LaneId::Core,
            CoreLane,
            CapabilityProfile::strict([HostCapability::StorageRead, HostCapability::EventEmit]),
        );
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        let receipt = kernel.execute_tx(
            &sample_block(),
            &sample_tx(LaneId::Core, 200_000),
            &mut state,
        );

        assert!(!receipt.success);
        assert_eq!(
            receipt.error,
            Some(KernelError::CapabilityDenied {
                lane: LaneId::Core,
                capability: HostCapability::StorageWrite,
            })
        );
    }

    #[test]
    fn canonical_receipt_includes_replay_and_trace_hashes() {
        let mut lanes = LaneRegistry::default();
        lanes.register(LaneId::Core, CoreLane);
        let kernel = CoreKernel::new(FuelSchedule::default(), lanes);

        let mut state = MemoryState::default();
        let canonical = kernel.execute_tx_canonical(
            &sample_block(),
            &sample_tx(LaneId::Core, 200_000),
            &mut state,
        );

        assert_eq!(canonical.status, CanonicalStatus::Success);
        assert_eq!(canonical.event_count, 1);
        assert_ne!(canonical.replay_hash, [0u8; 32]);
        assert_ne!(canonical.execution_trace_hash, [0u8; 32]);
        assert!(
            canonical
                .security_flags
                .contains(&SecurityFlag::CapabilityGatedHost)
        );
    }

    #[test]
    fn journal_checkpoint_and_rollback_work() {
        let mut journal = StateJournal::new();
        journal.begin_transaction();
        journal.put(LaneId::Core, b"a".to_vec(), b"1".to_vec());
        let checkpoint = journal.checkpoint();
        journal.put(LaneId::Core, b"a".to_vec(), b"2".to_vec());
        journal
            .rollback(checkpoint)
            .expect("rollback to checkpoint should succeed");

        let mut state = MemoryState::default();
        journal.commit(&mut state);
        assert_eq!(state.get(b"a"), Some(b"1".to_vec()));
    }

    #[test]
    fn read_state_prefers_journal_overlay() {
        let mut state = MemoryState::default();
        state.set(b"counter".to_vec(), b"1".to_vec());

        let block = sample_block();
        let tx = sample_tx(LaneId::Core, 200_000);
        let mut journal = StateJournal::new();
        journal.begin_transaction();
        let mut fuel = FuelMeter::new(200_000);
        let schedule = FuelSchedule::default();
        let mut env = ExecutionEnv {
            block: &block,
            tx: &tx,
            state: &mut state,
            journal: &mut journal,
            fuel: &mut fuel,
            schedule: &schedule,
        };

        env.write_state(b"counter".to_vec(), b"9".to_vec())
            .expect("write should succeed");
        assert_eq!(env.read_state(b"counter"), Some(b"9".to_vec()));
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
            sequence: 0,
            payload: b"bridge".to_vec(),
        };

        assert!(bus.enqueue(message.clone()).is_ok());
        assert!(matches!(
            bus.enqueue(message),
            Err(KernelError::DeterministicAbort { code: 77, .. })
        ));
    }

    #[test]
    fn enforces_cross_lane_causal_ordering() {
        let mut bus = CrossLaneBus::default();
        let first = CrossLaneMessage {
            version: 1,
            source_lane: LaneId::Evm,
            target_lane: LaneId::Wasm,
            tx_id: [7u8; 32],
            sequence: 0,
            payload: b"first".to_vec(),
        };
        let out_of_order = CrossLaneMessage {
            version: 1,
            source_lane: LaneId::Evm,
            target_lane: LaneId::Wasm,
            tx_id: [7u8; 32],
            sequence: 2,
            payload: b"third".to_vec(),
        };

        assert!(bus.enqueue(first).is_ok());
        assert!(matches!(
            bus.enqueue(out_of_order),
            Err(KernelError::DeterministicAbort { code: 78, .. })
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

    #[test]
    fn deterministic_planner_groups_conflict_free_intents() {
        let intents = vec![
            LaneExecutionIntent {
                lane: LaneId::Evm,
                tx_id: [1u8; 32],
                declared_reads: BTreeSet::from([b"lane/a/r1".to_vec()]),
                declared_writes: BTreeSet::from([b"lane/a/w1".to_vec()]),
            },
            LaneExecutionIntent {
                lane: LaneId::Wasm,
                tx_id: [2u8; 32],
                declared_reads: BTreeSet::from([b"lane/b/r1".to_vec()]),
                declared_writes: BTreeSet::from([b"lane/b/w1".to_vec()]),
            },
            LaneExecutionIntent {
                lane: LaneId::Move,
                tx_id: [3u8; 32],
                declared_reads: BTreeSet::from([b"lane/a/w1".to_vec()]),
                declared_writes: BTreeSet::new(),
            },
        ];

        let plan = plan_deterministic_batches(intents);
        assert_eq!(plan.serial_order, vec![[1u8; 32], [2u8; 32], [3u8; 32]]);
        assert_eq!(plan.parallel_batches.len(), 2);
        assert_eq!(plan.parallel_batches[0], vec![[1u8; 32], [2u8; 32]]);
        assert_eq!(plan.parallel_batches[1], vec![[3u8; 32]]);
    }
}
