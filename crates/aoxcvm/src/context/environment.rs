//! Execution environment context.

/// Canonical environment values selected by consensus/runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvironmentContext {
    pub chain_id: u64,
    pub network_id: u32,
}

impl EnvironmentContext {
    pub const fn new(chain_id: u64, network_id: u32) -> Self {
        Self {
            chain_id,
            network_id,
        }
    }
}
