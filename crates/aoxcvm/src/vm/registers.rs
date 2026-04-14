//! Minimal register surface for deterministic arithmetic paths.

/// VM register file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VmRegisters {
    accumulator: i64,
    zero_flag: bool,
}

impl VmRegisters {
    /// Updates the accumulator and derives flag state.
    pub fn set_accumulator(&mut self, value: i64) {
        self.accumulator = value;
        self.zero_flag = value == 0;
    }

    /// Returns accumulator value.
    pub const fn accumulator(self) -> i64 {
        self.accumulator
    }

    /// Zero flag derived from the accumulator.
    pub const fn zero_flag(self) -> bool {
        self.zero_flag
    }
}

#[cfg(test)]
mod tests {
    use super::VmRegisters;

    #[test]
    fn zero_flag_tracks_accumulator() {
        let mut regs = VmRegisters::default();
        regs.set_accumulator(0);
        assert!(regs.zero_flag());
        regs.set_accumulator(7);
        assert!(!regs.zero_flag());
    }
}
