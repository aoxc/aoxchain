use crate::errors::AoxcvmError;
use crate::result::Result;
use crate::vm::machine::MachineState;

pub fn step(state: &mut MachineState, gas_cost: u64, authority_cost: u32, max_gas: u64, max_authority: u32) -> Result<()> {
    state.gas_used = state.gas_used.saturating_add(gas_cost);
    state.authority_used = state.authority_used.saturating_add(authority_cost);
    state.pc = state.pc.saturating_add(1);

    if state.gas_used > max_gas {
        return Err(AoxcvmError::LimitExceeded("gas budget exceeded"));
    }
    if state.authority_used > max_authority {
        return Err(AoxcvmError::LimitExceeded("authority budget exceeded"));
    }
    Ok(())
}
