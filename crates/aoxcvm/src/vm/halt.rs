//! Halt conditions for deterministic VM termination.

/// Canonical reasons for VM halting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltReason {
    /// Program reached an explicit stop/halt instruction.
    Explicit,
    /// Program counter moved beyond the code boundary.
    EndOfCode,
    /// Runtime requested rollback semantics.
    Revert,
}
