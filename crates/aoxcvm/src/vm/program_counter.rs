//! Deterministic program-counter utilities for the VM kernel.

/// Deterministic program-counter state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramCounter {
    value: usize,
    code_len: usize,
}

impl ProgramCounter {
    /// Creates a program counter bound to a specific code length.
    pub const fn new(code_len: usize) -> Self {
        Self { value: 0, code_len }
    }

    /// Returns the current instruction offset.
    pub const fn value(self) -> usize {
        self.value
    }

    /// Returns the code length the counter is bounded by.
    pub const fn code_len(self) -> usize {
        self.code_len
    }

    /// Returns true when the counter points at or beyond the end of code.
    pub const fn is_halted(self) -> bool {
        self.value >= self.code_len
    }

    /// Advances by one instruction.
    ///
    /// Returns `false` if the counter is already halted.
    pub fn step(&mut self) -> bool {
        self.jump(self.value.saturating_add(1))
    }

    /// Jumps to an absolute instruction offset inside the code range.
    ///
    /// Returns `false` if `target` is outside bounds.
    pub fn jump(&mut self, target: usize) -> bool {
        if target > self.code_len {
            return false;
        }

        self.value = target;
        true
    }

    /// Resets the program counter to the first instruction.
    pub fn reset(&mut self) {
        self.value = 0;
    }
}

impl Default for ProgramCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::ProgramCounter;

    #[test]
    fn step_and_jump_are_bounds_checked() {
        let mut pc = ProgramCounter::new(3);
        assert_eq!(pc.value(), 0);
        assert!(pc.step());
        assert_eq!(pc.value(), 1);
        assert!(pc.jump(3));
        assert!(pc.is_halted());
        assert!(!pc.jump(4));
    }
}
