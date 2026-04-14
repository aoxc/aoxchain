//! Execution transition envelope for one deterministic VM step.

use crate::vm::{signals::VmSignal, traps::VmTrap};

/// One state transition emitted after an instruction dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepTransition {
    pub pc_before: usize,
    pub pc_after: usize,
    pub gas_delta: u64,
    pub signal: VmSignal,
}

impl StepTransition {
    /// Constructs a successful transition that continues execution.
    pub const fn continue_with(pc_before: usize, pc_after: usize, gas_delta: u64) -> Self {
        Self {
            pc_before,
            pc_after,
            gas_delta,
            signal: VmSignal::Continue,
        }
    }

    /// Constructs a trapping transition.
    pub const fn trap(pc: usize, gas_delta: u64, trap: VmTrap) -> Self {
        Self {
            pc_before: pc,
            pc_after: pc,
            gas_delta,
            signal: VmSignal::Trap(trap),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StepTransition;
    use crate::vm::traps::VmTrap;

    #[test]
    fn trap_transition_freezes_pc() {
        let t = StepTransition::trap(5, 2, VmTrap::OutOfGas);
        assert_eq!(t.pc_before, 5);
        assert_eq!(t.pc_after, 5);
    }
}
