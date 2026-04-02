//! Origin and actor execution context.

/// Origin/caller/callee tuple bound to one execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OriginContext {
    pub caller: [u8; 32],
    pub callee: [u8; 32],
    pub origin: [u8; 32],
    pub transferred_value: u128,
}

impl OriginContext {
    pub const fn new(
        caller: [u8; 32],
        callee: [u8; 32],
        origin: [u8; 32],
        transferred_value: u128,
    ) -> Self {
        Self {
            caller,
            callee,
            origin,
            transferred_value,
        }
    }
}
