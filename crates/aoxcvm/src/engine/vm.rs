//! Minimal deterministic VM state model for the phase-3 core prototype.

/// Canonical VM state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmState {
    pc: usize,
    halted: bool,
    reverted: bool,
    stack: Vec<i64>,
}

impl VmState {
    pub fn new() -> Self {
        Self {
            pc: 0,
            halted: false,
            reverted: false,
            stack: Vec::new(),
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    pub fn advance_pc(&mut self, amount: usize) {
        self.pc = self.pc.saturating_add(amount);
    }

    pub fn halt(&mut self) {
        self.halted = true;
    }

    pub fn revert(&mut self) {
        self.reverted = true;
        self.halted = true;
    }

    pub fn halted(&self) -> bool {
        self.halted
    }

    pub fn reverted(&self) -> bool {
        self.reverted
    }

    pub fn push(&mut self, value: i64) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Option<i64> {
        self.stack.pop()
    }

    pub fn stack(&self) -> &[i64] {
        &self.stack
    }
}

impl Default for VmState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::VmState;

    #[test]
    fn stack_and_pc_transitions_are_explicit() {
        let mut vm = VmState::new();
        assert_eq!(vm.pc(), 0);
        vm.advance_pc(3);
        assert_eq!(vm.pc(), 3);

        vm.push(10);
        vm.push(20);
        assert_eq!(vm.pop(), Some(20));
        assert_eq!(vm.stack(), &[10]);
    }
}
