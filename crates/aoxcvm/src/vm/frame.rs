//! Call-frame representation for the deterministic VM core.

/// A single VM frame captured on the call stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    return_pc: usize,
    locals_base: usize,
}

impl Frame {
    /// Creates a frame with explicit return program counter and local base.
    pub const fn new(return_pc: usize, locals_base: usize) -> Self {
        Self {
            return_pc,
            locals_base,
        }
    }

    /// Program counter to continue at after returning from this frame.
    pub const fn return_pc(self) -> usize {
        self.return_pc
    }

    /// Base stack index where this frame's locals start.
    pub const fn locals_base(self) -> usize {
        self.locals_base
    }
}
