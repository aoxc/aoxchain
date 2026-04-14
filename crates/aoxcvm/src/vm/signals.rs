//! Step-level VM execution signals.

use crate::vm::{halt::HaltReason, traps::VmTrap};

/// Result of a single dispatch step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmSignal {
    Continue,
    Halt(HaltReason),
    Trap(VmTrap),
}

impl VmSignal {
    /// Returns true when execution can continue.
    pub const fn can_continue(self) -> bool {
        matches!(self, Self::Continue)
    }
}
