use crate::gas::Gas;
use crate::vm_kind::VmKind;

/// Immutable per-block execution context shared by all lanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockContext {
    pub chain_id: u64,
    pub block_number: u64,
    pub block_timestamp: u64,
    pub proposer: [u8; 32],
    pub base_fee: u128,
}

impl BlockContext {
    /// Constructs a new immutable block context.
    pub const fn new(
        chain_id: u64,
        block_number: u64,
        block_timestamp: u64,
        proposer: [u8; 32],
        base_fee: u128,
    ) -> Self {
        Self {
            chain_id,
            block_number,
            block_timestamp,
            proposer,
            base_fee,
        }
    }
}

/// Canonical transaction envelope routed into a target execution lane.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxContext {
    pub tx_hash: [u8; 32],
    pub sender: Vec<u8>,
    pub vm_kind: VmKind,
    pub nonce: Option<u64>,
    pub gas_limit: Gas,
    pub max_fee_per_gas: u128,
    pub payload: Vec<u8>,
    pub signature: Vec<u8>,
}

impl TxContext {
    /// Performs host-level sanity checks shared across all lanes.
    pub fn validate_basic(&self) -> Result<(), crate::error::AovmError> {
        if self.sender.is_empty() {
            return Err(crate::error::AovmError::InvalidTransaction("sender is empty"));
        }
        if self.payload.is_empty() {
            return Err(crate::error::AovmError::InvalidTransaction("payload is empty"));
        }
        if self.gas_limit == 0 {
            return Err(crate::error::AovmError::InvalidTransaction("gas limit is zero"));
        }
        Ok(())
    }
}
