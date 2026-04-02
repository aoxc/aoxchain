//! Deterministic execution limits for context validation.

/// Deterministic execution guardrails selected by protocol/runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterminismLimits {
    pub max_call_depth: u16,
    pub max_gas_limit: u64,
    pub min_spec_version: u32,
}

impl Default for DeterminismLimits {
    fn default() -> Self {
        Self {
            max_call_depth: 64,
            max_gas_limit: 1_000_000_000,
            min_spec_version: 1,
        }
    }
}
