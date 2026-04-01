use crate::engine::executor::execute;
use crate::policy::vm_policy::VmPolicy;
use crate::result::Result;
use crate::state::overlay::StateOverlay;
use crate::tx::envelope::TransactionEnvelope;

pub fn simulate(tx: &TransactionEnvelope, policy: &VmPolicy) -> Result<u64> {
    let mut overlay = StateOverlay::default();
    let state = execute(tx, policy, &mut overlay)?;
    Ok(state.gas_used)
}
