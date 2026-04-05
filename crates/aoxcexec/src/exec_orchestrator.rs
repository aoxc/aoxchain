#[derive(Debug, Clone)]
pub struct DeterministicLane {
    lane_id: String,
}

impl DeterministicLane {
    #[must_use]
    pub fn new(lane_id: impl Into<String>) -> Self {
        Self {
            lane_id: lane_id.into(),
        }
    }
}

impl ExecutionLane for DeterministicLane {
    fn lane_id(&self) -> &str {
        &self.lane_id
    }

    fn validate_payload(
        &self,
        _context: &ExecutionContext,
        payload: &ExecutionPayload,
    ) -> Result<(), ReceiptFailure> {
        if payload.lane_id != self.lane_id {
            return Err(ReceiptFailure::LaneUnavailable(payload.lane_id.clone()));
        }
        Ok(())
    }

    fn estimate_intrinsic_gas(
        &self,
        policy: &LanePolicy,
        payload: &ExecutionPayload,
    ) -> Result<Gas, ExecutionError> {
        let payload_bytes_gas = policy
            .gas_per_byte
            .checked_mul(payload.data.len() as Gas)
            .ok_or(ExecutionError::ArithmeticOverflow)?;
        policy
            .base_gas
            .checked_add(payload_bytes_gas)
            .ok_or(ExecutionError::ArithmeticOverflow)
    }

    fn execute(
        &self,
        _context: &ExecutionContext,
        payload: &ExecutionPayload,
        pre_state: &dyn StateStore,
    ) -> Result<LaneExecutionOutput, ReceiptFailure> {
        let nonce_key = engine::sender_nonce_key(payload.sender, payload.nonce);
        if pre_state.get(&nonce_key).is_some() {
            return Err(ReceiptFailure::NonceGap {
                expected: payload.nonce.saturating_add(1),
                got: payload.nonce,
            });
        }

        let state_key = engine::state_key(&payload.lane_id, &payload.sender, payload.nonce);
        let write_set = WriteSet {
            writes: vec![
                WriteOperation {
                    key: state_key,
                    value: payload.data.clone(),
                },
                WriteOperation {
                    key: nonce_key,
                    value: payload.nonce.to_le_bytes().to_vec(),
                },
            ],
        };
        let trace = vec![
            format!(
                "lane={} payload_type={:?}",
                payload.lane_id, payload.payload_type
            ),
            format!("sender={}", hex::encode(payload.sender)),
            format!("nonce={} bytes={}", payload.nonce, payload.data.len()),
        ];
        Ok(LaneExecutionOutput { trace, write_set })
    }

    fn verify_result(&self, output: &LaneExecutionOutput) -> Result<(), ReceiptFailure> {
        let trace_len: usize = output.trace.iter().map(String::len).sum();
        if trace_len > 2_048 {
            return Err(ReceiptFailure::TraceTooLarge {
                bytes: trace_len,
                max: 2_048,
            });
        }
        Ok(())
    }

    fn commit_changes(
        &self,
        output: &LaneExecutionOutput,
        state: &mut dyn StateStore,
    ) -> Result<StateDiff, ExecutionError> {
        let mut diff = Vec::with_capacity(output.write_set.writes.len());
        for write in &output.write_set.writes {
            diff.push(StateDiffEntry {
                key: write.key.clone(),
                before: state.get(&write.key),
                after: Some(write.value.clone()),
            });
        }
        state.apply_write_set(&output.write_set);
        Ok(StateDiff { entries: diff })
    }
}

pub trait ExecutionOrchestrator {
    fn execute_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<BatchExecutionOutcome, ExecutionError>;
}

/// Audit-oriented deterministic orchestrator that enforces lane policy before
/// handing execution to downstream runtimes.
pub struct DeterministicOrchestrator {
    lane_registry: LaneRegistry,
    lanes: BTreeMap<String, Box<dyn ExecutionLane + Send + Sync>>,
    initial_state: InMemoryStateStore,
}

impl Default for DeterministicOrchestrator {
    fn default() -> Self {
        Self::new(
            default_lane_registry(),
            default_lanes(),
            InMemoryStateStore::default(),
        )
    }
}

impl DeterministicOrchestrator {
    #[must_use]
    pub fn new(
        lane_registry: LaneRegistry,
        lanes: impl IntoIterator<Item = Box<dyn ExecutionLane + Send + Sync>>,
        initial_state: InMemoryStateStore,
    ) -> Self {
        let lanes = lanes
            .into_iter()
            .map(|lane| (lane.lane_id().to_string(), lane))
            .collect();
        Self {
            lane_registry,
            lanes,
            initial_state,
        }
    }

    pub fn summarize_batch(
        &self,
        context: &ExecutionContext,
        payloads: &[ExecutionPayload],
    ) -> Result<ExecutionBatchSummary, ExecutionError> {
        self.execute_batch(context, payloads)
            .map(|result| result.summary)
    }
}


#[path = "engine/mod.rs"]
mod engine;

pub use engine::{default_lane_registry, default_lanes};

/// Compatibility alias retained for existing users that still instantiate the
/// old placeholder orchestrator.
pub type PlaceholderOrchestrator = DeterministicOrchestrator;
