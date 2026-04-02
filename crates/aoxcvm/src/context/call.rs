//! Call depth execution context.

/// Canonical depth and call-frame controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallContext {
    pub depth: u16,
}

impl CallContext {
    pub const fn new(depth: u16) -> Self {
        Self { depth }
    }
}
