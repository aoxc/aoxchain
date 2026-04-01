use crate::policy::execution::enforce_execution_policy;
use crate::policy::vm_policy::VmPolicy;
use crate::result::Result;
use crate::state::overlay::StateOverlay;
use crate::tx::envelope::TransactionEnvelope;
use crate::vm::machine::MachineState;
use crate::vm::transition::step;

pub fn execute(tx: &TransactionEnvelope, policy: &VmPolicy, overlay: &mut StateOverlay) -> Result<MachineState> {
    enforce_execution_policy(policy, 1, 1)?;

    let mut machine = MachineState::default();
    step(&mut machine, 10, 1, tx.max_gas, tx.max_authority)?;

    overlay.write(tx.tx_hash);
    Ok(machine)
}
