//! Canonical VM trap categories.

/// Deterministic trap taxonomy for VM execution errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmTrap {
    StackUnderflow,
    StackOverflow,
    InvalidJump,
    OutOfGas,
    ArithmeticOverflow,
}
