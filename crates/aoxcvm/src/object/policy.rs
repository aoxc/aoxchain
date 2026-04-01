#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutationPolicy {
    pub upgradable: bool,
    pub transferable: bool,
    pub requires_capability: bool,
}
