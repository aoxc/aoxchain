//! Native execution lanes hosted by the shared AOXC runtime.

pub mod cardano;
pub mod evm;
pub mod sui_move;
pub mod wasm;

use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::vm_kind::VmKind;

/// Common execution contract implemented by every native lane.
pub trait VirtualMachine {
    /// Returns the VM identity served by the executor.
    fn kind(&self) -> VmKind;

    /// Performs pre-execution lane-specific validation.
    fn validate_tx(
        &self,
        state: &dyn HostStateView,
        block: &BlockContext,
        tx: &TxContext,
    ) -> Result<(), AovmError>;

    /// Executes a transaction against mutable host state.
    fn execute_tx(
        &self,
        state: &mut dyn HostStateView,
        block: &BlockContext,
        tx: &TxContext,
    ) -> Result<ExecutionReceipt, AovmError>;

    /// Executes a read-only query against lane state.
    fn query(
        &self,
        state: &dyn HostStateView,
        block: &BlockContext,
        payload: &[u8],
    ) -> Result<Vec<u8>, AovmError>;
}
