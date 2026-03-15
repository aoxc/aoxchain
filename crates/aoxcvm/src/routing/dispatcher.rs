use crate::context::{BlockContext, TxContext};
use crate::error::AovmError;
use crate::host::receipt::ExecutionReceipt;
use crate::host::state::HostStateView;
use crate::lanes::VirtualMachine;
use crate::vm_kind::VmKind;

/// Routes a canonical transaction into the correct native lane.
pub struct Dispatcher<'a> {
    pub evm: &'a dyn VirtualMachine,
    pub sui_move: &'a dyn VirtualMachine,
    pub wasm: &'a dyn VirtualMachine,
    pub cardano: &'a dyn VirtualMachine,
}

impl<'a> Dispatcher<'a> {
    /// Executes a transaction by dispatching on `vm_kind`.
    pub fn execute(
        &self,
        state: &mut dyn HostStateView,
        block: &BlockContext,
        tx: &TxContext,
    ) -> Result<ExecutionReceipt, AovmError> {
        match tx.vm_kind {
            VmKind::Evm => self.evm.execute_tx(state, block, tx),
            VmKind::SuiMove => self.sui_move.execute_tx(state, block, tx),
            VmKind::Wasm => self.wasm.execute_tx(state, block, tx),
            VmKind::Cardano => self.cardano.execute_tx(state, block, tx),
        }
    }
}
